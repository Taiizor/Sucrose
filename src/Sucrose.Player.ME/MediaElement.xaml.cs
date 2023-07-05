using System.Windows;
using System.Windows.Media;
using System.Windows.Threading;
using SEDST = Skylark.Enum.DuplicateScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEST = Skylark.Enum.ScreenType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPMEEH = Sucrose.Player.ME.Event.Handler;
using SPMEHMEH = Sucrose.Player.ME.Helper.MediaElementHelper;
using SPMEMI = Sucrose.Player.ME.Manage.Internal;
using SPSEH = Sucrose.Player.Shared.Event.Handler;
using SPSHS = Sucrose.Player.Shared.Helper.Source;
using SSEDT = Sucrose.Space.Enum.DisplayType;
using SSEST = Sucrose.Space.Enum.StretchType;
using SWE = Skylark.Wing.Engine;

namespace Sucrose.Player.ME
{
    /// <summary>
    /// Interaction logic for MediaElement.xaml
    /// </summary>
    public sealed partial class MediaElement : Window
    {
        private readonly DispatcherTimer Timer = new();

        public MediaElement()
        {
            InitializeComponent();

            Content = SPMEMI.MediaPlayer;

            SPMEMI.MediaPlayer.Source = SPSHS.GetSource(SMMI.EngineSettingManager.GetSetting(SMC.Video, @""));

            Timer.Tick += new EventHandler(Timer_Tick);
            Timer.Interval = new TimeSpan(0, 0, 1);
            Timer.Start();

            SPMEMI.MediaPlayer.MediaOpened += SPMEEH.MediaPlayerOpened;
            SPMEMI.MediaPlayer.MediaFailed += SPMEEH.MediaPlayerFailed;
            SPMEMI.MediaPlayer.MediaEnded += SPMEEH.MediaPlayerEnded;

            Closing += (s, e) => SPMEMI.MediaPlayer.Close();
            Loaded += (s, e) => SPSEH.WindowLoaded(this);

            SPMEHMEH.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));

            SPMEHMEH.Play();
        }

        private void Timer_Tick(object sender, EventArgs e)
        {
            SPMEHMEH.SetLoop(SMMI.EngineSettingManager.GetSetting(SMC.Loop, true));

            SPMEHMEH.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));

            SSEST Stretch = SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SSEST.Fill);

            if ((int)Stretch < Enum.GetValues(typeof(SSEST)).Length)
            {
                SPMEMI.MediaPlayer.Stretch = (Stretch)Stretch;
            }

            Uri Video = SPSHS.GetSource(SMMI.EngineSettingManager.GetSetting(SMC.Video, new Uri(@"", UriKind.RelativeOrAbsolute)));

            if (SPMEMI.MediaPlayer.Source != Video)
            {
                SPMEMI.MediaPlayer.Source = Video;
            }
        }

        private void MediaElement_ContentRendered(object sender, EventArgs e)
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