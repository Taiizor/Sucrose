using CefSharp;
using System.IO;
using System.Windows;
using System.Windows.Interop;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using Application = System.Windows.Application;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;

namespace Sucrose.Shared.Engine.CefSharp.Extension
{
    internal static class Screenshot
    {
        public static string Capture()
        {
            Window Window = Application.Current.MainWindow;

            RenderTargetBitmap Bitmap = new((int)Window.Width, (int)Window.Height, 96, 96, PixelFormats.Default);

            Bitmap.Render(Window);

            JpegBitmapEncoder Encoder = new()
            {
                QualityLevel = 100
            };

            Encoder.Frames.Add(BitmapFrame.Create(Bitmap));

            MemoryStream Stream = new();

            Encoder.Save(Stream);

            byte[] Bytes = Stream.ToArray();
            string Base64 = Convert.ToBase64String(Bytes);

            return $"data:image/jpeg;base64,{Base64}";
        }

        public static async Task<string> Capture2()
        {
            SSECSMI.CefEngine.ExecuteScriptAsync(SSEHS.GetScreenshot());

            JavascriptResponse Response = await SSECSMI.CefEngine.EvaluateScriptAsync(@"html2canvas(document.body).then(canvas => { return canvas.toDataURL(""image/jpeg""); });");

            if (Response.Success)
            {
                return Response.Result as string;
            }
            else
            {
                return string.Empty;
            }
        }

        private static string ToBase64(this InteropBitmap InteropBitmap)
        {
            BitmapSource BitmapSource = InteropBitmap;

            if (BitmapSource != null)
            {
                MemoryStream Stream = new();

                BitmapEncoder Encoder = new JpegBitmapEncoder();

                Encoder.Frames.Add(BitmapFrame.Create(BitmapSource));
                Encoder.Save(Stream);

                byte[] Bytes = Stream.ToArray();
                string Base64 = Convert.ToBase64String(Bytes);

                return $"data:image/jpeg;base64,{Base64}";
            }
            else
            {
                throw new ArgumentException("Given InteropBitmap cannot be converted to BitmapSource.");
            }
        }
    }
}