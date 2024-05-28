using System.IO;

namespace Sucrose.XamlAnimatedGif.Decoding
{
    internal class GifImageData
    {
        public byte LzwMinimumCodeSize { get; set; }
        public long CompressedDataStartOffset { get; set; }

        private GifImageData()
        {
        }

        internal static async Task<GifImageData> ReadAsync(Stream stream)
        {
            GifImageData imgData = new();
            await imgData.ReadInternalAsync(stream).ConfigureAwait(false);
            return imgData;
        }

        private async Task ReadInternalAsync(Stream stream)
        {
            LzwMinimumCodeSize = (byte)stream.ReadByte();
            CompressedDataStartOffset = stream.Position;
            await GifHelpers.ConsumeDataBlocksAsync(stream).ConfigureAwait(false);
        }
    }
}