﻿using SEWTT = Skylark.Enum.WindowsThemeType;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SSLMI = Sucrose.Shared.Launcher.Manage.Internal;
//using SSLVDFB = Sucrose.Shared.Launcher.View.DarkFeedbackBox;
//using SSLVLFB = Sucrose.Shared.Launcher.View.LightFeedbackBox;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Feedback
    {
        public static void Command()
        {
            if (SSLMI.FeedbackBox)
            {
                SSLMI.FeedbackBox = false;

                switch (SSDMM.ThemeType)
                {
                    case SEWTT.Dark:
                        //SSLVDFB DarkFeedbackBox = new();
                        //DarkFeedbackBox.ShowDialog();
                        break;
                    default:
                        //SSLVLFB LightFeedbackBox = new();
                        //LightFeedbackBox.ShowDialog();
                        break;
                }

                SSLMI.FeedbackBox = true;
            }
        }
    }
}