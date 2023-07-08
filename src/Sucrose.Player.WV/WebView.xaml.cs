using Microsoft.Web.WebView2.Core;
using System.Windows;
using System.Windows.Threading;
using SEDST = Skylark.Enum.DuplicateScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEST = Skylark.Enum.ScreenType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPSEH = Sucrose.Player.Shared.Event.Handler;
using SPSHS = Sucrose.Player.Shared.Helper.Source;
using SPWVEH = Sucrose.Player.WV.Event.Handler;
using SPWVHWVH = Sucrose.Player.WV.Helper.WebViewHelper;
using SPWVMI = Sucrose.Player.WV.Manage.Internal;
using SSEDT = Sucrose.Space.Enum.DisplayType;
using SSEST = Sucrose.Space.Enum.StretchType;
using SWE = Skylark.Wing.Engine;

namespace Sucrose.Player.WV
{
    /// <summary>
    /// Interaction logic for WebView.xaml
    /// </summary>
    public sealed partial class WebView : Window
    {
        private readonly DispatcherTimer Timer = new();

        public WebView(string Video)
        {
            InitializeComponent();

            ContentRendered += (s, e) => SPSEH.ContentRendered(this);

            Content = SPWVMI.EdgePlayer;

            SPWVMI.Video = Video;

            Timer.Tick += new EventHandler(Timer_Tick);
            Timer.Interval = new TimeSpan(0, 0, 1);
            Timer.Start();

            SPWVMI.EdgePlayer.CoreWebView2InitializationCompleted += SPWVEH.EdgePlayerInitializationCompleted;

            Closing += (s, e) => SPWVMI.EdgePlayer.Dispose();
            Loaded += (s, e) => SPSEH.WindowLoaded(this);
        }

        private void CoreWebView2_DOMContentLoaded(object sender, CoreWebView2DOMContentLoadedEventArgs e)
        {
            throw new NotImplementedException();
        }

        private async void Timer_Tick(object sender, EventArgs e)
        {
            SPWVHWVH.SetLoop(SMMI.EngineSettingManager.GetSetting(SMC.Loop, true));

            SPWVHWVH.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));

            SSEST Stretch = SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SSEST.Fill);

            if ((int)Stretch < Enum.GetValues(typeof(SSEST)).Length)
            {
                SPWVHWVH.SetStretch(Stretch);
            }

//            if (SPWVMI.EdgePlayer.IsInitialized)
//            {
//                Uri Video = SPSHS.GetSource(SMMI.EngineSettingManager.GetSetting(SMC.Video, @""));

//                if (SPSHS.GetExtension(Video))
//                {
//                    if (SPWVMI.EdgePlayer.Source != Video)
//                    {
//                        SPWVMI.EdgePlayer.Source = Video;
//                    }
//                }
//                else
//                {
//                    string Source = await SPWVHWVH.GetVideo();
//                    string Local = Video.ToString();

//#if NET48_OR_GREATER
//                    Source = Source.Substring(1, Source.Length - 2);
//#else
//                    Source = Source[1..^1];
//#endif

//                    if (Source != Local)
//                    {
//                        SPWVHWVH.SetVideo(Local);
//                    }
//                }
//            }
        }

        private void WebView_ContentRendered(object sender, EventArgs e)
        {
            switch (SMMI.EngineSettingManager.GetSetting(SMC.DisplayType, SSEDT.Screen))
            {
                case SSEDT.Expand:
                    SWE.WallpaperWindow(this, SMMI.EngineSettingManager.GetSetting(SMC.ExpandScreenType, SEEST.Default), SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SEST.DisplayBound));
                    break;
                case SSEDT.Duplicate:
                    SWE.WallpaperWindow(this, SMMI.EngineSettingManager.GetSetting(SMC.DuplicateScreenType, SEDST.Default), SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SEST.DisplayBound));
                    break;
                default:
                    SWE.WallpaperWindow(this, SMMI.EngineSettingManager.GetSettingStable(SMC.ScreenIndex, 0), SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SEST.DisplayBound));
                    break;
            }
        }
    }
}