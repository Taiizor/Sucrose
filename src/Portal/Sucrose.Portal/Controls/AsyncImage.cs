using System.IO;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media.Imaging;

namespace Sucrose.Portal.Controls
{
    public class AsyncImage : Image, IDisposable
    {
        public static readonly DependencyProperty ImagePathProperty = DependencyProperty.Register(nameof(ImagePath), typeof(string), typeof(AsyncImage), new PropertyMetadata(async (o, e) => await ((AsyncImage)o).LoadImageAsync((string)e.NewValue)));

        public string ImagePath
        {
            get => (string)GetValue(ImagePathProperty);
            set => SetValue(ImagePathProperty, value);
        }

        private async Task LoadImageAsync(string imagePath)
        {
            Source = await Task.Run(() =>
            {
                using FileStream stream = File.OpenRead(imagePath);

                BitmapImage bitmap = new();

                bitmap.BeginInit();

                bitmap.CacheOption = BitmapCacheOption.OnLoad;
                bitmap.StreamSource = stream;
                bitmap.DecodePixelWidth = 360;

                bitmap.EndInit();
                bitmap.Freeze();

                return bitmap;
            });
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}