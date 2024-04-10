using Microsoft.Web.WebView2.Core;
using System.IO;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;

namespace Sucrose.Shared.Engine.WebView.Extension
{
    internal static class Screenshot
    {
        public static async Task<string> Capture()
        {
            MemoryStream Stream = new();

            await SSEWVMI.WebEngine.CoreWebView2.CapturePreviewAsync(CoreWebView2CapturePreviewImageFormat.Jpeg, Stream);

            return Convert.ToBase64String(Stream.ToArray());
        }
    }
}