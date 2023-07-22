using CefSharp;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;

namespace Sucrose.Shared.Engine.CefSharp.Helper
{
    internal static class Evaluate
    {
        public static async Task<string> ScriptString(string Script)
        {
            string Result = string.Empty;
            object Object = await ScriptObject(Script);

            if (Object != null)
            {
                return Object.ToString();
            }

            return Result;
        }

        public static async Task<object> ScriptObject(string Script)
        {
            object Result = null;
            JavascriptResponse Response = await ScriptResponse(Script);

            if (Response.Success)
            {
                return Response.Result;
            }

            return Result;
        }

        public static async Task<JavascriptResponse> ScriptResponse(string Script)
        {
            JavascriptResponse Response;

            if (SSECSMI.CefEngine.CanExecuteJavascriptInMainFrame)
            {
                Response = await SSECSMI.CefEngine.EvaluateScriptAsync(Script);
            }
            else
            {
                IFrame Frame = SSECSMI.CefEngine.GetMainFrame();

                Response = await Frame.EvaluateScriptAsync(Script);
            }

            return Response;
        }
    }
}