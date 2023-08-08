using System.IO;
using System.Net.Cache;
using System.Windows.Media.Imaging;

namespace Sucrose.Portal.Extension
{
    internal class ImageLoader : IDisposable
    {
        private readonly List<Stream> ImageStreams = new();

        public BitmapImage Load(string ImagePath)
        {
            FileStream ImageStream = new(ImagePath, FileMode.Open, FileAccess.Read);

            ImageStreams.Add(ImageStream);

            BitmapImage BitmapImage = new()
            {
                UriCachePolicy = new(RequestCacheLevel.NoCacheNoStore),
                CreateOptions = BitmapCreateOptions.IgnoreImageCache,
                CacheOption = BitmapCacheOption.None
            };

            BitmapImage.BeginInit();

            BitmapImage.StreamSource = ImageStream;

            BitmapImage.EndInit();

            return BitmapImage;
        }

        public void Dispose()
        {
            ImageStreams.ForEach(ImageStream => ImageStream.Dispose());

            ImageStreams.Clear();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}