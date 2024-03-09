using System.Text;
using Microsoft.UI.Xaml.Media.Imaging;
using Windows.Storage.Streams;

namespace Rhythm.Helpers;
public static class BitmapHelper
{
    public static async Task<BitmapImage> GetBitmapAsync(byte[] data)
    {
        var bitmapImage = new BitmapImage();

        using (var stream = new InMemoryRandomAccessStream())
        {
            using (var writer = new DataWriter(stream))
            {
                writer.WriteBytes(data);
                await writer.StoreAsync();
                await writer.FlushAsync();
                writer.DetachStream();
            }
            stream.Seek(0);
            await bitmapImage.SetSourceAsync(stream);
        }

        return bitmapImage;
    }

    public static async Task<string> GetBase64Async(byte[] data)
    {

        return await Task.Run(() => Convert.ToBase64String(data));
    }
}
