using System.Drawing.Imaging;
using System.IO;
using SBMM = Sucrose.Backgroundog.Manage.Manager;

namespace Sucrose.Backgroundog.Extension
{
    internal class Thumbnail
    {
        public static string Create(Stream stream)
        {
            using MemoryStream Stream = new();

            Stream.Seek(0, SeekOrigin.Begin);
            stream.CopyTo(Stream);

            if (!SBMM.Windows11_OrGreater)
            {
                using Bitmap Image = new(Stream);

                if (PixelAlpha(Image, 0, 0))
                {
                    return CropImage(Image, 34, 1, 233, 233);
                }
            }

            byte[] Array = Stream.ToArray();

            return Convert.ToBase64String(Array);
        }

        private static bool PixelAlpha(Bitmap Image, int X, int Y)
        {
            return Image.GetPixel(X, Y).A == 0;
        }

        private static string CropImage(Bitmap Image, int X, int Y, int Width, int Height)
        {
            Rectangle Rect = new(X, Y, Width, Height);

            using Bitmap CroppedBitmap = new(Rect.Width, Rect.Height, Image.PixelFormat);

            Graphics Graphic = Graphics.FromImage(CroppedBitmap);
            Graphic.DrawImage(Image, 0, 0, Rect, GraphicsUnit.Pixel);

            using MemoryStream Stream = new();

            CroppedBitmap.Save(Stream, ImageFormat.Png);

            byte[] ByteImage = Stream.ToArray();

            return Convert.ToBase64String(ByteImage);
        }
    }
}