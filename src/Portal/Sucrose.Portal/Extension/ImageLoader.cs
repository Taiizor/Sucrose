using System.IO;
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

            BitmapImage BitmapImage = new();

            BitmapImage.BeginInit();

            BitmapImage.CacheOption = BitmapCacheOption.OnLoad;
            BitmapImage.StreamSource = ImageStream;

            BitmapImage.EndInit();

            return BitmapImage;
        }

        public void Dispose()
        {
            foreach (Stream ImageStream in ImageStreams)
            {
                ImageStream.Dispose();
            }

            ImageStreams.Clear();

            GC.SuppressFinalize(this);
        }
    }
}