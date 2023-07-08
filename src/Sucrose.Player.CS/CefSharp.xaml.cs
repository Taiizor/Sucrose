using System.Windows;
using System.Windows.Threading;
using SEDST = Skylark.Enum.DuplicateScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEST = Skylark.Enum.ScreenType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPCSEH = Sucrose.Player.CS.Event.Handler;
using SPCSHCSH = Sucrose.Player.CS.Helper.CefSharpHelper;
using SPCSMI = Sucrose.Player.CS.Manage.Internal;
using SPSEH = Sucrose.Player.Shared.Event.Handler;
using SPSHS = Sucrose.Player.Shared.Helper.Source;
using SSEDT = Sucrose.Space.Enum.DisplayType;
using SSEST = Sucrose.Space.Enum.StretchType;
using SWE = Skylark.Wing.Engine;

namespace Sucrose.Player.CS
{
    /// <summary>
    /// Interaction logic for CefSharp.xaml
    /// </summary>
    public sealed partial class CefSharp : Window
    {
        private readonly DispatcherTimer Timer = new();

        public CefSharp(string Video)
        {
            InitializeComponent();

            ContentRendered += (s, e) => SPSEH.ContentRendered(this);

            SPCSMI.CefPlayer.MenuHandler = new CustomContextMenuHandler();

            Content = SPCSMI.CefPlayer;

            SPCSMI.CefPlayer.Address = SPSHS.GetSource(Video).ToString();
            //SPCSMI.CefPlayer.Address = @"http://www.bokowsky.net/de/knowledge-base/video/videos/big_buck_bunny_240p.ogg"; //.webm - .mp4 - .ogg

            SPCSMI.CefPlayer.BrowserSettings = SPCSMI.CefSettings;

            Timer.Tick += new EventHandler(Timer_Tick);
            Timer.Interval = new TimeSpan(0, 0, 1);
            Timer.Start();

            SPCSMI.CefPlayer.FrameLoadEnd += SPCSEH.CefPlayerFrameLoadEnd;
            SPCSMI.CefPlayer.Loaded += SPCSEH.CefPlayerLoaded;

            Closing += (s, e) => SPCSMI.CefPlayer.Dispose();
            Loaded += (s, e) => SPSEH.WindowLoaded(this);
        }

        private void Timer_Tick(object sender, EventArgs e)
        {
            SPCSHCSH.SetLoop(SMMI.EngineSettingManager.GetSetting(SMC.Loop, true));

            SPCSHCSH.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));

            SSEST Stretch = SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SSEST.Fill);

            if ((int)Stretch < Enum.GetValues(typeof(SSEST)).Length)
            {
                SPCSHCSH.SetStretch(Stretch);
            }
        }

        private void CefSharp_ContentRendered(object sender, EventArgs e)
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