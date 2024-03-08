// This source file's Lempel-Ziv-Welch algorithm is derived from Chromium's Android GifPlayer
// as seen here (https://github.com/chromium/chromium/blob/master/third_party/gif_player/src/jp/tomorrowkey/android/gifplayer)
// Licensed under the Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// Copyright (C) 2015 The Gifplayer Authors. All Rights Reserved.

// The rest of the source file is licensed under MIT License.
// Copyright (C) 2018 Jumar A. Macato, All Rights Reserved.

using Avalonia.Media.Imaging;
using System;
using System.Buffers;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using static Avalonia.Gif.Extensions.StreamExtensions;

namespace Avalonia.Gif.Decoding
{
    public sealed class GifDecoder : IDisposable
    {
        private static readonly ReadOnlyMemory<byte> G87AMagic
            = "GIF87a"u8.ToArray().AsMemory();

        private static readonly ReadOnlyMemory<byte> G89AMagic
            = "GIF89a"u8.ToArray().AsMemory();

        private static readonly ReadOnlyMemory<byte> NetscapeMagic
            = "NETSCAPE2.0"u8.ToArray().AsMemory();

        private static readonly TimeSpan FrameDelayThreshold = TimeSpan.FromMilliseconds(10);
        private static readonly TimeSpan FrameDelayDefault = TimeSpan.FromMilliseconds(100);
        private static readonly GifColor TransparentColor = new(0, 0, 0, 0);
        private static readonly int MaxTempBuf = 768;
        private static readonly int MaxStackSize = 4096;
        private static readonly int MaxBits = 4097;

        private readonly Stream _fileStream;
        private readonly CancellationToken _currentCtsToken;
        private readonly bool _hasFrameBackups;

        private int _gctSize, _bgIndex, _prevFrame = -1, _backupFrame = -1;
        private bool _gctUsed;

        private GifRect _gifDimensions;

        // private ulong _globalColorTable;
        private readonly int _backBufferBytes;
        private GifColor[] _bitmapBackBuffer;

        private short[] _prefixBuf;
        private byte[] _suffixBuf;
        private byte[] _pixelStack;
        private byte[] _indexBuf;
        private byte[] _backupFrameIndexBuf;
        private volatile bool _hasNewFrame;

        public GifHeader Header { get; private set; }

        public readonly List<GifFrame> Frames = new();

        public PixelSize Size => new(Header.Dimensions.Width, Header.Dimensions.Height);

        public GifDecoder(Stream fileStream, CancellationToken currentCtsToken)
        {
            _fileStream = fileStream;
            _currentCtsToken = currentCtsToken;

            ProcessHeaderData();
            ProcessFrameData();

            Header.IterationCount = Header.Iterations switch
            {
                -1 => new GifRepeatBehavior { Count = 1 },
                0 => new GifRepeatBehavior { LoopForever = true },
                > 0 => new GifRepeatBehavior { Count = Header.Iterations },
                _ => Header.IterationCount
            };

            int pixelCount = _gifDimensions.TotalPixels;

            _hasFrameBackups = Frames
                .Any(f => f.FrameDisposalMethod == FrameDisposal.Restore);

            _bitmapBackBuffer = new GifColor[pixelCount];
            _indexBuf = new byte[pixelCount];

            if (_hasFrameBackups)
            {
                _backupFrameIndexBuf = new byte[pixelCount];
            }

            _prefixBuf = new short[MaxStackSize];
            _suffixBuf = new byte[MaxStackSize];
            _pixelStack = new byte[MaxStackSize + 1];

            _backBufferBytes = pixelCount * Marshal.SizeOf(typeof(GifColor));
        }

        public void Dispose()
        {
            Frames.Clear();

            _bitmapBackBuffer = null;
            _prefixBuf = null;
            _suffixBuf = null;
            _pixelStack = null;
            _indexBuf = null;
            _backupFrameIndexBuf = null;
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        private int PixCoord(int x, int y)
        {
            return x + (y * _gifDimensions.Width);
        }

        static readonly (int Start, int Step)[] Pass =
        {
            (0, 8),
            (4, 8),
            (2, 4),
            (1, 2)
        };

        private void ClearImage()
        {
            Array.Fill(_bitmapBackBuffer, TransparentColor);
            //ClearArea(_gifDimensions);

            _prevFrame = -1;
            _backupFrame = -1;
        }

        public void RenderFrame(int fIndex, WriteableBitmap writeableBitmap, bool forceClear = false)
        {
            if (_currentCtsToken.IsCancellationRequested)
            {
                return;
            }

            if (fIndex < 0 | fIndex >= Frames.Count)
            {
                return;
            }

            if (_prevFrame == fIndex)
            {
                return;
            }

            if (fIndex == 0 || forceClear || fIndex < _prevFrame)
            {
                ClearImage();
            }

            DisposePreviousFrame();

            _prevFrame++;

            // render intermediate frame
            for (int idx = _prevFrame; idx < fIndex; ++idx)
            {
                GifFrame prevFrame = Frames[idx];

                if (prevFrame.FrameDisposalMethod == FrameDisposal.Restore)
                {
                    continue;
                }

                if (prevFrame.FrameDisposalMethod == FrameDisposal.Background)
                {
                    ClearArea(prevFrame.Dimensions);
                    continue;
                }

                RenderFrameAt(idx, writeableBitmap);
            }

            RenderFrameAt(fIndex, writeableBitmap);
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        private void RenderFrameAt(int idx, WriteableBitmap writeableBitmap)
        {
            byte[] tmpB = ArrayPool<byte>.Shared.Rent(MaxTempBuf);

            GifFrame curFrame = Frames[idx];
            DecompressFrameToIndexBuffer(curFrame, _indexBuf, tmpB);

            if (_hasFrameBackups & curFrame.ShouldBackup)
            {
                Buffer.BlockCopy(_indexBuf, 0, _backupFrameIndexBuf, 0, curFrame.Dimensions.TotalPixels);
                _backupFrame = idx;
            }

            DrawFrame(curFrame, _indexBuf);

            _prevFrame = idx;
            _hasNewFrame = true;

            using Platform.ILockedFramebuffer lockedBitmap = writeableBitmap.Lock();
            WriteBackBufToFb(lockedBitmap.Address);

            ArrayPool<byte>.Shared.Return(tmpB);
        }


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        private void DrawFrame(GifFrame curFrame, Memory<byte> frameIndexSpan)
        {
            GifColor[] activeColorTable =
                curFrame.IsLocalColorTableUsed ? curFrame.LocalColorTable : Header.GlobarColorTable;

            int cX = curFrame.Dimensions.X;
            int cY = curFrame.Dimensions.Y;
            int cH = curFrame.Dimensions.Height;
            int cW = curFrame.Dimensions.Width;
            byte tC = curFrame.TransparentColorIndex;
            bool hT = curFrame.HasTransparency;

            if (curFrame.IsInterlaced)
            {
                for (int i = 0; i < 4; i++)
                {
                    (int Start, int Step) = Pass[i];
                    int y = Start;
                    while (y < cH)
                    {
                        DrawRow(y);
                        y += Step;
                    }
                }
            }
            else
            {
                for (int i = 0; i < cH; i++)
                {
                    DrawRow(i);
                }
            }

            //for (var row = 0; row < cH; row++)
            void DrawRow(int row)
            {
                // Get the starting point of the current row on frame's index stream.
                int indexOffset = row * cW;

                // Get the target backbuffer offset from the frames coords.
                int targetOffset = PixCoord(cX, row + cY);
                int len = _bitmapBackBuffer.Length;

                for (int i = 0; i < cW; i++)
                {
                    byte indexColor = frameIndexSpan.Span[indexOffset + i];

                    if (activeColorTable == null || targetOffset >= len ||
                        indexColor > activeColorTable.Length)
                    {
                        return;
                    }

                    if (!(hT & indexColor == tC))
                    {
                        _bitmapBackBuffer[targetOffset] = activeColorTable[indexColor];
                    }

                    targetOffset++;
                }
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        private void DisposePreviousFrame()
        {
            if (_prevFrame == -1)
            {
                return;
            }

            GifFrame prevFrame = Frames[_prevFrame];

            switch (prevFrame.FrameDisposalMethod)
            {
                case FrameDisposal.Background:
                    ClearArea(prevFrame.Dimensions);
                    break;
                case FrameDisposal.Restore:
                    if (_hasFrameBackups && _backupFrame != -1)
                    {
                        DrawFrame(Frames[_backupFrame], _backupFrameIndexBuf);
                    }
                    else
                    {
                        ClearArea(prevFrame.Dimensions);
                    }

                    break;
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        private void ClearArea(GifRect area)
        {
            for (int y = 0; y < area.Height; y++)
            {
                int targetOffset = PixCoord(area.X, y + area.Y);
                for (int x = 0; x < area.Width; x++)
                {
                    _bitmapBackBuffer[targetOffset + x] = TransparentColor;
                }
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        private void DecompressFrameToIndexBuffer(GifFrame curFrame, Span<byte> indexSpan, byte[] tempBuf)
        {
            _fileStream.Position = curFrame.LzwStreamPosition;
            int totalPixels = curFrame.Dimensions.TotalPixels;

            // Initialize GIF data stream decoder.
            int dataSize = curFrame.LzwMinCodeSize;
            int clear = 1 << dataSize;
            int endOfInformation = clear + 1;
            int available = clear + 2;
            int oldCode = -1;
            int codeSize = dataSize + 1;
            int codeMask = (1 << codeSize) - 1;

            for (int code = 0; code < clear; code++)
            {
                _prefixBuf[code] = 0;
                _suffixBuf[code] = (byte)code;
            }

            // Decode GIF pixel stream.
            int bits, first, top, pixelIndex;
            int datum = bits = first = top = pixelIndex = 0;

            while (pixelIndex < totalPixels)
            {
                int blockSize = _fileStream.ReadBlock(tempBuf);

                if (blockSize == 0)
                {
                    break;
                }

                int blockPos = 0;

                while (blockPos < blockSize)
                {
                    datum += tempBuf[blockPos] << bits;
                    blockPos++;

                    bits += 8;

                    while (bits >= codeSize)
                    {
                        // Get the next code.
                        int code = datum & codeMask;
                        datum >>= codeSize;
                        bits -= codeSize;

                        // Interpret the code
                        if (code == clear)
                        {
                            // Reset decoder.
                            codeSize = dataSize + 1;
                            codeMask = (1 << codeSize) - 1;
                            available = clear + 2;
                            oldCode = -1;
                            continue;
                        }

                        // Check for explicit end-of-stream
                        if (code == endOfInformation)
                        {
                            return;
                        }

                        if (oldCode == -1)
                        {
                            indexSpan[pixelIndex++] = _suffixBuf[code];
                            oldCode = code;
                            first = code;
                            continue;
                        }

                        int inCode = code;
                        if (code >= available)
                        {
                            _pixelStack[top++] = (byte)first;
                            code = oldCode;

                            if (top == MaxBits)
                            {
                                ThrowException();
                            }
                        }

                        while (code >= clear)
                        {
                            if (code >= MaxBits || code == _prefixBuf[code])
                            {
                                ThrowException();
                            }

                            _pixelStack[top++] = _suffixBuf[code];
                            code = _prefixBuf[code];

                            if (top == MaxBits)
                            {
                                ThrowException();
                            }
                        }

                        first = _suffixBuf[code];
                        _pixelStack[top++] = (byte)first;

                        // Add new code to the dictionary
                        if (available < MaxStackSize)
                        {
                            _prefixBuf[available] = (short)oldCode;
                            _suffixBuf[available] = (byte)first;
                            available++;

                            if ((available & codeMask) == 0 && available < MaxStackSize)
                            {
                                codeSize++;
                                codeMask += available;
                            }
                        }

                        oldCode = inCode;

                        // Drain the pixel stack.
                        do
                        {
                            indexSpan[pixelIndex++] = _pixelStack[--top];
                        } while (top > 0);
                    }
                }
            }

            while (pixelIndex < totalPixels)
            {
                indexSpan[pixelIndex++] = 0; // clear missing pixels
            }

            static void ThrowException()
            {
                throw new LzwDecompressionException();
            }
        }

        /// <summary>
        /// Directly copies the <see cref="GifColor"/> struct array to a bitmap IntPtr.
        /// </summary>
        private void WriteBackBufToFb(IntPtr targetPointer)
        {
            if (_currentCtsToken.IsCancellationRequested)
            {
                return;
            }

            if (!(_hasNewFrame & _bitmapBackBuffer != null))
            {
                return;
            }

            unsafe
            {
                fixed (void* src = &_bitmapBackBuffer[0])
                {
                    Buffer.MemoryCopy(src, targetPointer.ToPointer(), (uint)_backBufferBytes,
                        (uint)_backBufferBytes);
                }

                _hasNewFrame = false;
            }
        }

        /// <summary>
        /// Processes GIF Header.
        /// </summary>
        private void ProcessHeaderData()
        {
            Stream str = _fileStream;
            byte[] tmpB = ArrayPool<byte>.Shared.Rent(MaxTempBuf);
            Span<byte> tempBuf = tmpB.AsSpan();

            int _ = str.Read(tmpB, 0, 6);

            if (!tempBuf[..3].SequenceEqual(G87AMagic[..3].Span))
            {
                throw new InvalidGifStreamException("Not a GIF stream.");
            }

            if (!(tempBuf[..6].SequenceEqual(G87AMagic.Span) |
                  tempBuf[..6].SequenceEqual(G89AMagic.Span)))
            {
                throw new InvalidGifStreamException("Unsupported GIF Version: " +
                                                    Encoding.ASCII.GetString(tempBuf[..6].ToArray()));
            }

            ProcessScreenDescriptor(tmpB);

            Header = new GifHeader
            {
                Dimensions = _gifDimensions,
                HasGlobalColorTable = _gctUsed,
                // GlobalColorTableCacheID = _globalColorTable,
                GlobarColorTable = ProcessColorTable(ref str, tmpB, _gctSize),
                GlobalColorTableSize = _gctSize,
                BackgroundColorIndex = _bgIndex,
                HeaderSize = _fileStream.Position
            };

            ArrayPool<byte>.Shared.Return(tmpB);
        }

        /// <summary>
        /// Parses colors from file stream to target color table.
        /// </summary> 
        private static GifColor[] ProcessColorTable(ref Stream stream, byte[] rawBufSpan, int nColors)
        {
            int nBytes = 3 * nColors;
            GifColor[] target = new GifColor[nColors];

            int n = stream.Read(rawBufSpan, 0, nBytes);

            if (n < nBytes)
            {
                throw new InvalidOperationException("Wrong color table bytes.");
            }

            int i = 0, j = 0;

            while (i < nColors)
            {
                byte r = rawBufSpan[j++];
                byte g = rawBufSpan[j++];
                byte b = rawBufSpan[j++];
                target[i++] = new GifColor(r, g, b);
            }

            return target;
        }

        /// <summary>
        /// Parses screen and other GIF descriptors. 
        /// </summary>
        private void ProcessScreenDescriptor(byte[] tempBuf)
        {
            ushort width = _fileStream.ReadUShortS(tempBuf);
            ushort height = _fileStream.ReadUShortS(tempBuf);

            byte packed = _fileStream.ReadByteS(tempBuf);

            _gctUsed = (packed & 0x80) != 0;
            _gctSize = 2 << (packed & 7);
            _bgIndex = _fileStream.ReadByteS(tempBuf);

            _gifDimensions = new GifRect(0, 0, width, height);
            _fileStream.Skip(1);
        }

        /// <summary>
        /// Parses all frame data.
        /// </summary>
        private void ProcessFrameData()
        {
            _fileStream.Position = Header.HeaderSize;

            byte[] tempBuf = ArrayPool<byte>.Shared.Rent(MaxTempBuf);

            bool terminate = false;
            int curFrame = 0;

            Frames.Add(new GifFrame());

            do
            {
                BlockTypes blockType = (BlockTypes)_fileStream.ReadByteS(tempBuf);

                switch (blockType)
                {
                    case BlockTypes.Empty:
                        break;

                    case BlockTypes.Extension:
                        ProcessExtensions(ref curFrame, tempBuf);
                        break;

                    case BlockTypes.ImageDescriptor:
                        ProcessImageDescriptor(ref curFrame, tempBuf);
                        _fileStream.SkipBlocks(tempBuf);
                        break;

                    case BlockTypes.Trailer:
                        Frames.RemoveAt(Frames.Count - 1);
                        terminate = true;
                        break;

                    default:
                        _fileStream.SkipBlocks(tempBuf);
                        break;
                }

                // Break the loop when the stream is not valid anymore.
                if (_fileStream.Position >= _fileStream.Length & terminate == false)
                {
                    throw new InvalidProgramException("Reach the end of the filestream without trailer block.");
                }
            } while (!terminate);

            ArrayPool<byte>.Shared.Return(tempBuf);
        }

        /// <summary>
        /// Parses GIF Image Descriptor Block.
        /// </summary>
        private void ProcessImageDescriptor(ref int curFrame, byte[] tempBuf)
        {
            Stream str = _fileStream;
            GifFrame currentFrame = Frames[curFrame];

            // Parse frame dimensions.
            ushort frameX = str.ReadUShortS(tempBuf);
            ushort frameY = str.ReadUShortS(tempBuf);
            ushort frameW = str.ReadUShortS(tempBuf);
            ushort frameH = str.ReadUShortS(tempBuf);

            frameW = (ushort)Math.Min(frameW, _gifDimensions.Width - frameX);
            frameH = (ushort)Math.Min(frameH, _gifDimensions.Height - frameY);

            currentFrame.Dimensions = new GifRect(frameX, frameY, frameW, frameH);

            // Unpack interlace and lct info.
            byte packed = str.ReadByteS(tempBuf);
            currentFrame.IsInterlaced = (packed & 0x40) != 0;
            currentFrame.IsLocalColorTableUsed = (packed & 0x80) != 0;
            currentFrame.LocalColorTableSize = (int)Math.Pow(2, (packed & 0x07) + 1);

            if (currentFrame.IsLocalColorTableUsed)
            {
                currentFrame.LocalColorTable =
                    ProcessColorTable(ref str, tempBuf, currentFrame.LocalColorTableSize);
            }

            currentFrame.LzwMinCodeSize = str.ReadByteS(tempBuf);
            currentFrame.LzwStreamPosition = str.Position;

            curFrame += 1;
            Frames.Add(new GifFrame());
        }

        /// <summary>
        /// Parses GIF Extension Blocks.
        /// </summary>
        private void ProcessExtensions(ref int curFrame, byte[] tempBuf)
        {
            ExtensionType extType = (ExtensionType)_fileStream.ReadByteS(tempBuf);

            switch (extType)
            {
                case ExtensionType.GraphicsControl:

                    _fileStream.ReadBlock(tempBuf);
                    GifFrame currentFrame = Frames[curFrame];
                    byte packed = tempBuf[0];

                    currentFrame.FrameDisposalMethod = (FrameDisposal)((packed & 0x1c) >> 2);

                    if (currentFrame.FrameDisposalMethod is not FrameDisposal.Restore
                        and not FrameDisposal.Background)
                    {
                        currentFrame.ShouldBackup = true;
                    }

                    currentFrame.HasTransparency = (packed & 1) != 0;

                    currentFrame.FrameDelay =
                        TimeSpan.FromMilliseconds(SpanToShort(tempBuf.AsSpan(1)) * 10);

                    if (currentFrame.FrameDelay <= FrameDelayThreshold)
                    {
                        currentFrame.FrameDelay = FrameDelayDefault;
                    }

                    currentFrame.TransparentColorIndex = tempBuf[3];
                    break;

                case ExtensionType.Application:
                    int blockLen = _fileStream.ReadBlock(tempBuf);
                    Span<byte> _ = tempBuf.AsSpan(0, blockLen);
                    Span<byte> blockHeader = tempBuf.AsSpan(0, NetscapeMagic.Length);

                    if (blockHeader.SequenceEqual(NetscapeMagic.Span))
                    {
                        int count = 1;

                        while (count > 0)
                        {
                            count = _fileStream.ReadBlock(tempBuf);
                        }

                        ushort iterationCount = SpanToShort(tempBuf.AsSpan(1));

                        Header.Iterations = iterationCount;
                    }
                    else
                    {
                        _fileStream.SkipBlocks(tempBuf);
                    }

                    break;

                default:
                    _fileStream.SkipBlocks(tempBuf);
                    break;
            }
        }
    }
}