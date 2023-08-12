using System.IO;
using System.Net.Cache;
using System.Windows.Media.Imaging;
using SPMI = Sucrose.Portal.Manage.Internal;

namespace Sucrose.Portal.Extension
{
    internal class ImageLoader : IDisposable
    {
        private FileStream ImageStream = null;

        public BitmapImage Load(string ImagePath)
        {
            if (!SPMI.Images.ContainsKey(ImagePath))
            {
                SPMI.Images.Add(ImagePath, new()
                {
                    UriCachePolicy = new(RequestCacheLevel.NoCacheNoStore),
                    CreateOptions = BitmapCreateOptions.IgnoreImageCache,
                    CacheOption = BitmapCacheOption.None
                });

                ImageStream = new(ImagePath, FileMode.Open, FileAccess.Read);

                SPMI.Images[ImagePath].BeginInit();

                SPMI.Images[ImagePath].StreamSource = ImageStream;
                SPMI.Images[ImagePath].DecodePixelWidth = 360;

                SPMI.Images[ImagePath].EndInit();
            }

            return SPMI.Images[ImagePath];
        }

        public void Remove(string ImagePath)
        {
            SPMI.Images.Remove(ImagePath);
        }

        public void Clear()
        {
            SPMI.Images.Clear();
        }

        public void Dispose()
        {
            ImageStream?.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}