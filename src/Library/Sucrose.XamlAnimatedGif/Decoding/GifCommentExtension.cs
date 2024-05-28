using System.IO;

namespace Sucrose.XamlAnimatedGif.Decoding
{
    internal class GifCommentExtension : GifExtension
    {
        internal const int ExtensionLabel = 0xFE;

        public string Text { get; private set; }

        private GifCommentExtension()
        {
        }

        internal override GifBlockKind Kind => GifBlockKind.SpecialPurpose;

        internal static async Task<GifCommentExtension> ReadAsync(Stream stream)
        {
            GifCommentExtension comment = new();
            await comment.ReadInternalAsync(stream).ConfigureAwait(false);
            return comment;
        }

        private async Task ReadInternalAsync(Stream stream)
        {
            // Note: at this point, the label (0xFE) has already been read

            byte[] bytes = await GifHelpers.ReadDataBlocksAsync(stream).ConfigureAwait(false);
            if (bytes != null)
            {
                Text = GifHelpers.GetString(bytes);
            }
        }
    }
}