using System.Drawing.Imaging;
using System.IO;
using SBMI = Sucrose.Backgroundog.Manage.Internal;

namespace Sucrose.Backgroundog.Extension
{
    internal static class Thumbnail
    {
        public static string Create(Stream Stream)
        {
            using MemoryStream MemoryStream = new();

            MemoryStream.Seek(0, SeekOrigin.Begin);
            Stream.CopyTo(MemoryStream);

            if (!SBMI.Windows11_OrGreater)
            {
                using Bitmap Image = new(MemoryStream);

                if (PixelAlpha(Image, 0, 0))
                {
                    return CropImage(Image, 34, 1, 233, 233);
                }
            }

            byte[] Array = MemoryStream.ToArray();

            MemoryStream.Flush();

            return Convert.ToBase64String(Array);
        }

        private static bool PixelAlpha(Bitmap Image, int X, int Y)
        {
            return Image.GetPixel(X, Y).A == 0;
        }

        private static string CropImage(Bitmap Image, int X, int Y, int Width, int Height)
        {
            Rectangle Rect = new(X, Y, Width, Height);

            using Bitmap CroppedImage = new(Rect.Width, Rect.Height, Image.PixelFormat);

            Graphics Graphic = Graphics.FromImage(CroppedImage);
            Graphic.DrawImage(Image, 0, 0, Rect, GraphicsUnit.Pixel);

            using MemoryStream Stream = new();

            CroppedImage.Save(Stream, ImageFormat.Png);

            byte[] ByteImage = Stream.ToArray();

            Stream.Flush();

            return Convert.ToBase64String(ByteImage);
        }
    }
}