using System.IO;
using System.Windows.Interop;
using System.Windows.Media.Imaging;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;

namespace Sucrose.Shared.Engine.CefSharp.Extension
{
    internal static class Screenshot
    {
        public static async Task<string> Capture()
        {
            InteropBitmap Bitmap = await SSECSMI.CefEngine.TakeScreenshot();

            return Bitmap.ToBase64();

            //SSECSMI.CefEngine.ExecuteScriptAsync(SSEHS.GetScreenshot());

            //JavascriptResponse Response = await SSECSMI.CefEngine.EvaluateScriptAsync(@"html2canvas(document.body).then(canvas => { return canvas.toDataURL(""image/jpeg""); });");

            //if (Response.Success)
            //{
            //    return Response.Result as string;
            //}
            //else
            //{
            //    return string.Empty;
            //}
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

                // Data URL oluşturma
                return $"data:image/jpeg;base64,{Base64}";
            }
            else
            {
                throw new ArgumentException("Given InteropBitmap cannot be converted to BitmapSource.");
            }
        }
    }
}