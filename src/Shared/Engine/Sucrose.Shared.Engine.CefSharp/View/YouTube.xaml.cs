﻿using Microsoft.Win32;
using System.Windows;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSECSEYT = Sucrose.Shared.Engine.CefSharp.Event.YouTube;
using SSECSHCCM = Sucrose.Shared.Engine.CefSharp.Handler.CustomContextMenu;
using SSECSHYT = Sucrose.Shared.Engine.CefSharp.Helper.YouTube;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEEH = Sucrose.Shared.Engine.Event.Handler;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHR = Sucrose.Shared.Engine.Helper.Run;
using SSEHV = Sucrose.Shared.Engine.Helper.Volume;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SMMB = Sucrose.Manager.Manage.Backgroundog;
using SMMCB = Sucrose.Memory.Manage.Constant.Backgroundog;

namespace Sucrose.Shared.Engine.CefSharp.View
{
    /// <summary>
    /// Interaction logic for YouTube.xaml
    /// </summary>
    public sealed partial class YouTube : Window, IDisposable
    {
        public YouTube(string YouTube)
        {
            InitializeComponent();

            SystemEvents.DisplaySettingsChanged += (s, e) => SSEEH.DisplaySettingsChanged(this);

            ContentRendered += (s, e) => SSEEH.ContentRendered(this);

            SSECSMI.CefEngine.MenuHandler = new SSECSHCCM();

            Content = SSECSMI.CefEngine;

            SSECSMI.YouTube = YouTube;

            SSECSMI.CefEngine.BrowserSettings = SSECSMI.CefSettings;

            SSEMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SSEMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SSEMI.GeneralTimer.Start();

            SSECSMI.CefEngine.IsBrowserInitializedChanged += SSECSEYT.CefEngineInitializedChanged;
            SSECSMI.CefEngine.FrameLoadEnd += SSECSEYT.CefEngineFrameLoadEnd;
            SSECSMI.CefEngine.Loaded += SSECSEYT.CefEngineLoaded;

            Closing += (s, e) => SSECSMI.CefEngine.Dispose();
            Loaded += (s, e) => SSEEH.WindowLoaded(this);

            SSEHV.Start();
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            if (SSEMI.Initialized)
            {
                Dispose();

                SSEHR.Control();

                SSECSHYT.First();

                SSECSHYT.SetLoop(SSEHD.GetLoop());

                SSECSHYT.SetVolume(SSEHD.GetVolume());

                SSECSHYT.SetShuffle(SSEHD.GetShuffle());

                if (SMMB.PausePerformance)
                {
                    SSECSHYT.Pause();

                    SSEMI.PausePerformance = true;
                }
                else if (SSEMI.PausePerformance)
                {
                    SSECSHYT.Play();

                    SSEMI.PausePerformance = false;
                }
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}