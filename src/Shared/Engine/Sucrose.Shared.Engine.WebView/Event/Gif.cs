using Microsoft.Web.WebView2.Core;
using SELLT = Skylark.Enum.LevelLogType;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEWVHG = Sucrose.Shared.Engine.WebView.Helper.Gif;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SMME = Sucrose.Manager.Manage.Engine;
using SMMCE = Sucrose.Memory.Manage.Constant.Engine;
using SMMMCE = Sucrose.Memory.Manage.Constant.Engine;

namespace Sucrose.Shared.Engine.WebView.Event
{
    internal static class Gif
    {
        public static void WebEngineProcessFailed(object sender, CoreWebView2ProcessFailedEventArgs e)
        {
            SMMI.WebViewLiveLogManager.Log(SELLT.Fatal, $"Reason: {e.Reason}");
            SMMI.WebViewLiveLogManager.Log(SELLT.Fatal, $"Exit Code: {e.ExitCode}");
            SMMI.WebViewLiveLogManager.Log(SELLT.Fatal, $"Process Failed Kind: {e.ProcessFailedKind}");
            SMMI.WebViewLiveLogManager.Log(SELLT.Fatal, $"Process Description: {e.ProcessDescription}");
            SMMI.WebViewLiveLogManager.Log(SELLT.Fatal, $"Failure Source Module Path: {e.FailureSourceModulePath}");

            if (e.FrameInfosForFailedProcess != null && e.FrameInfosForFailedProcess.Any())
            {
                foreach (CoreWebView2FrameInfo FrameInfo in e.FrameInfosForFailedProcess)
                {
                    SMMI.WebViewLiveLogManager.Log(SELLT.Fatal, $"Failed Process; Frame ID: {FrameInfo.FrameId}, Frame Kind: {FrameInfo.FrameKind}, Name: {FrameInfo.Name}, Source: {FrameInfo.Source}");
                }
            }
        }

        public static void WebEngineDOMContentLoaded(object sender, CoreWebView2DOMContentLoadedEventArgs e)
        {
            SSEWVHG.Load();

            SSEMI.Initialized = true;
        }

        public static void WebEngineInitializationCompleted(object sender, CoreWebView2InitializationCompletedEventArgs e)
        {
            SSEWVMI.WebEngine.CoreWebView2.ProcessFailed += WebEngineProcessFailed;

            SSEWVMI.WebEngine.CoreWebView2.Settings.UserAgent = SMMM.UserAgent;

            Uri Gif = SSEHS.GetSource(SSEWVMI.Gif);

            string Path = SSEHS.GetGifContentPath();

            SSEHS.WriteGifContent(Path, Gif);

            SSEWVMI.WebEngine.Source = SSEHS.GetSource(Path);

            SSEWVMI.WebEngine.CoreWebView2.DOMContentLoaded += WebEngineDOMContentLoaded;

            if (SMME.DeveloperMode)
            {
                SSEWVMI.WebEngine.CoreWebView2.OpenDevToolsWindow();
            }
        }
    }
}