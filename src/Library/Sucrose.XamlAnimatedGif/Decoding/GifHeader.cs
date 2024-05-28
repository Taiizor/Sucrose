using System.IO;

namespace Sucrose.XamlAnimatedGif.Decoding
{
    internal class GifHeader : GifBlock
    {
        public string Signature { get; private set; }
        public string Version { get; private set; }
        public GifLogicalScreenDescriptor LogicalScreenDescriptor { get; private set; }

        private GifHeader()
        {
        }

        internal override GifBlockKind Kind => GifBlockKind.Other;

        internal static async Task<GifHeader> ReadAsync(Stream stream)
        {
            GifHeader header = new();
            await header.ReadInternalAsync(stream).ConfigureAwait(false);
            return header;
        }

        private async Task ReadInternalAsync(Stream stream)
        {
            Signature = await GifHelpers.ReadStringAsync(stream, 3).ConfigureAwait(false);
            if (Signature != "GIF")
            {
                throw GifHelpers.InvalidSignatureException(Signature);
            }

            Version = await GifHelpers.ReadStringAsync(stream, 3).ConfigureAwait(false);
            if (Version is not "87a" and not "89a")
            {
                throw GifHelpers.UnsupportedVersionException(Version);
            }

            LogicalScreenDescriptor = await GifLogicalScreenDescriptor.ReadAsync(stream).ConfigureAwait(false);
        }
    }
}