using Microsoft.WindowsAPICodePack.Shell;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;
using System.Net.Cache;
using System.Windows.Media.Imaging;

namespace Sucrose.Portal.Extension
{
    internal class ThumbnailLoader : IDisposable
    {
        public string SourcePath { get; set; } = null;

        public Uri SourceUri { get; set; } = null;

        public BitmapImage Load(string SourcePath)
        {
            BitmapImage Image;

            if (File.Exists(SourcePath))
            {
                this.SourcePath = SourcePath;
                SourceUri = new(SourcePath);

                try
                {
                    ShellFile Shell = ShellFile.FromFilePath(SourcePath);
                    Bitmap Thumbnail = Shell.Thumbnail.ExtraLargeBitmap;

                    Image = Convert(Thumbnail);

                    Dispose();
                }
                catch
                {
                    Image = Default();

                    Dispose();
                }
            }
            else
            {
                Image = Default();

                Dispose();
            }

            return Image;
        }

        public async Task<BitmapImage> LoadAsync(string ImagePath)
        {
            return await Task.Run(() => Load(ImagePath));
        }

        private BitmapImage Default()
        {
            BitmapImage Image = new();

            Image.BeginInit();

            Image.UriCachePolicy = new(RequestCacheLevel.BypassCache);
            Image.CacheOption = BitmapCacheOption.OnLoad;

            Image.UriSource = new Uri("pack://application:,,,/Assets/Theme/Default.jpg", UriKind.RelativeOrAbsolute);

            SourcePath = null;

            Image.EndInit();

            return Image;
        }

        private BitmapImage Convert(Bitmap Bitmap)
        {
            using MemoryStream Stream = new();

            Bitmap.Save(Stream, ImageFormat.Jpeg);

            Stream.Seek(0, SeekOrigin.Begin);

            BitmapImage Image = new();

            Image.BeginInit();

            Image.UriCachePolicy = new(RequestCacheLevel.BypassCache);
            Image.CacheOption = BitmapCacheOption.OnLoad;

            Image.StreamSource = Stream;

            Image.EndInit();

            Image.Freeze();

            Image.StreamSource.Flush();

            Stream.Flush();
            Stream.Close();
            Stream.Dispose();

            return Image;
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }

        public async Task DisposeAsync()
        {
            await Task.Run(Dispose);
        }
    }
}