using System.IO;

namespace Sucrose.Shared.Space.Extension
{
    internal class ProgressStream(Stream Stream, long TotalBytes, Action<long, long, double> Progress) : Stream
    {
        public override bool CanWrite => Stream.CanWrite;

        public override bool CanRead => Stream.CanRead;

        public override bool CanSeek => Stream.CanSeek;

        public override long Length => Stream.Length;

        private long BytesTransferred;

        public override void Flush()
        {
            Stream.Flush();
        }

        public override long Position
        {
            get => Stream.Position;
            set => Stream.Position = value;
        }

        public override void SetLength(long value)
        {
            Stream.SetLength(value);
        }

        public override long Seek(long offset, SeekOrigin origin)
        {
            return Stream.Seek(offset, origin);
        }

        public override int Read(byte[] buffer, int offset, int count)
        {
            int bytesRead = Stream.Read(buffer, offset, count);

            BytesTransferred += bytesRead;

            Progress(BytesTransferred, TotalBytes, (double)BytesTransferred / TotalBytes * 100);

            return bytesRead;
        }

        public override void Write(byte[] buffer, int offset, int count)
        {
            Stream.Write(buffer, offset, count);

            BytesTransferred += count;

            Progress(BytesTransferred, TotalBytes, (double)BytesTransferred / TotalBytes * 100);
        }
    }
}