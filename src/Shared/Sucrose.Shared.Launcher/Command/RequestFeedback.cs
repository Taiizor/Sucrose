using SEWTT = Skylark.Enum.WindowsThemeType;
using SSDMMG = Sucrose.Shared.Dependency.Manage.Manager.General;
using SSLMI = Sucrose.Shared.Launcher.Manage.Internal;
//using SSLVDRFB = Sucrose.Shared.Launcher.View.DarkRequestFeedbackBox;
//using SSLVLRFB = Sucrose.Shared.Launcher.View.LightRequestFeedbackBox;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class RequestFeedback
    {
        public static void Command()
        {
            if (SSLMI.RequestFeedbackBox)
            {
                SSLMI.RequestFeedbackBox = false;

                switch (SSDMMG.ThemeType)
                {
                    case SEWTT.Dark:
                        //SSLVDRFB DarkRequestFeedbackBox = new();
                        //DarkRequestFeedbackBox.ShowDialog();
                        break;
                    default:
                        //SSLVLRFB LightRequestFeedbackBox = new();
                        //LightRequestFeedbackBox.ShowDialog();
                        break;
                }

                SSLMI.RequestFeedbackBox = true;
            }
        }
    }
}