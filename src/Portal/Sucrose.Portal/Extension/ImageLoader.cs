using System.IO;
using System.Net.Cache;
using System.Windows.Media.Imaging;
using SPMI = Sucrose.Portal.Manage.Internal;

namespace Sucrose.Portal.Extension
{
    internal class ImageLoader : IDisposable
    {
        private string ImagePath { get; set; }

        public BitmapImage Load(string ImagePath)
        {
            this.ImagePath = ImagePath;

            if (!SPMI.Images.ContainsKey(ImagePath))
            {
                SPMI.Images.Add(ImagePath, new()
                {
                    UriCachePolicy = new(RequestCacheLevel.NoCacheNoStore),
                    CreateOptions = BitmapCreateOptions.IgnoreImageCache,
                    CacheOption = BitmapCacheOption.None
                });

                if (!SPMI.ImageStream.ContainsKey(ImagePath))
                {
                    SPMI.ImageStream[ImagePath] = new(ImagePath, FileMode.Open, FileAccess.Read);
                }

                SPMI.Images[ImagePath].BeginInit();

                SPMI.Images[ImagePath].StreamSource = SPMI.ImageStream[ImagePath];
                //SPMI.Images[ImagePath].DecodePixelHeight = 160;
                SPMI.Images[ImagePath].DecodePixelWidth = 360;

                SPMI.Images[ImagePath].EndInit();
            }

            return SPMI.Images[ImagePath];
        }

        public async Task<BitmapImage> LoadAsync(string ImagePath)
        {
            return await Task.Run(() => Load(ImagePath));
        }

        public BitmapImage LoadOptimal(string ImagePath)
        {
            BitmapImage Image = new();

            using FileStream Stream = new(ImagePath, FileMode.Open, FileAccess.Read);

            Image.BeginInit();

            Image.UriCachePolicy = new(RequestCacheLevel.BypassCache);
            Image.CacheOption = BitmapCacheOption.OnLoad;
            //Image.DecodePixelHeight = 160;
            Image.DecodePixelWidth = 360;
            Image.StreamSource = Stream;

            Image.EndInit();

            Image.Freeze();

            Image.StreamSource.Flush();

            Stream.Flush();
            Stream.Close();
            Stream.Dispose();

            Dispose();

            return Image;
        }

        public async Task<BitmapImage> LoadOptimalAsync(string ImagePath)
        {
            return await Task.Run(() => LoadOptimal(ImagePath));
        }

        public void Remove(string ImagePath)
        {
            if (!string.IsNullOrEmpty(ImagePath))
            {
                SPMI.Images.Remove(ImagePath);
                SPMI.ImageStream.Remove(ImagePath);
            }
        }

        public async Task RemoveAsync(string ImagePath)
        {
            await Task.Run(() => Remove(ImagePath));
        }

        public void Clear()
        {
            SPMI.Images.Clear();
            SPMI.ImageStream.Clear();
        }

        public async Task ClearAsync()
        {
            await Task.Run(Clear);
        }

        public void Dispose()
        {
            if (!string.IsNullOrEmpty(ImagePath))
            {
                SPMI.ImageStream[ImagePath].Dispose();
                SPMI.Images[ImagePath].StreamSource.Dispose();
            }

            GC.Collect();
            GC.SuppressFinalize(this);
        }

        public async Task DisposeAsync()
        {
            await Task.Run(Dispose);
        }
    }
}