﻿using Microsoft.Win32;
using System.Windows;
using SMMB = Sucrose.Manager.Manage.Backgroundog;
using SSEEH = Sucrose.Shared.Engine.Event.Handler;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHR = Sucrose.Shared.Engine.Helper.Run;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEWVEG = Sucrose.Shared.Engine.WebView.Event.Gif;
using SSEWVHG = Sucrose.Shared.Engine.WebView.Helper.Gif;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;

namespace Sucrose.Shared.Engine.WebView.View
{
    /// <summary>
    /// Interaction logic for Gif.xaml
    /// </summary>
    public sealed partial class Gif : Window, IDisposable
    {
        public Gif(string Gif)
        {
            InitializeComponent();

            SystemEvents.DisplaySettingsChanged += (s, e) => SSEEH.DisplaySettingsChanged(this);

            ContentRendered += (s, e) => SSEEH.ContentRendered(this);

            Content = SSEWVMI.WebEngine;

            SSEWVMI.Gif = Gif;

            SSEMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SSEMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SSEMI.GeneralTimer.Start();

            SSEWVMI.WebEngine.CoreWebView2InitializationCompleted += SSEWVEG.WebEngineInitializationCompleted;

            Closing += (s, e) => SSEWVMI.WebEngine.Dispose();
            Loaded += (s, e) => SSEEH.WindowLoaded(this);
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            if (SSEMI.Initialized)
            {
                Dispose();

                SSEHR.Control();

                SSEWVHG.SetLoop(SSEHD.GetLoop());

                SSEWVHG.SetStretch(SSEHD.GetStretch());

                if (SMMB.PausePerformance)
                {
                    SSEWVHG.Pause();

                    SSEMI.PausePerformance = true;
                }
                else if (SSEMI.PausePerformance)
                {
                    SSEWVHG.Play();

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