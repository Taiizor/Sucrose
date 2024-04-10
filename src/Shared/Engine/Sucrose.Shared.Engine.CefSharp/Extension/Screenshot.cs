using CefSharp;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;

namespace Sucrose.Shared.Engine.CefSharp.Extension
{
    internal static class Screenshot
    {
        public static async Task<string> Capture()
        {
            SSECSMI.CefEngine.ExecuteScriptAsync(SSEHS.GetScreenshot());

            JavascriptResponse Response = await SSECSMI.CefEngine.EvaluateScriptAsync(@"html2canvas(document.body).then(canvas => { return canvas.toDataURL(""image/jpeg""); });");

            if (Response.Result != null)
            {
                return Response.Result as string;
            }
            else
            {
                return string.Empty;
            }
        }
    }
}