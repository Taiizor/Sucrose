﻿using Microsoft.Win32;
using System.Windows;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSEEH = Sucrose.Shared.Engine.Event.Handler;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHR = Sucrose.Shared.Engine.Helper.Run;
using SSEHV = Sucrose.Shared.Engine.Helper.Volume;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEWVEU = Sucrose.Shared.Engine.WebView.Event.Url;
using SSEWVHU = Sucrose.Shared.Engine.WebView.Helper.Url;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SMMB = Sucrose.Manager.Manage.Backgroundog;
using SMMCB = Sucrose.Memory.Manage.Constant.Backgroundog;

namespace Sucrose.Shared.Engine.WebView.View
{
    /// <summary>
    /// Interaction logic for Url.xaml
    /// </summary>
    public sealed partial class Url : Window, IDisposable
    {
        public Url(string Url)
        {
            InitializeComponent();

            SystemEvents.DisplaySettingsChanged += (s, e) => SSEEH.DisplaySettingsChanged(this);

            ContentRendered += (s, e) => SSEEH.ContentRendered(this);

            Content = SSEWVMI.WebEngine;

            SSEWVMI.Url = Url;

            SSEMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SSEMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SSEMI.GeneralTimer.Start();

            SSEWVMI.WebEngine.CoreWebView2InitializationCompleted += SSEWVEU.WebEngineInitializationCompleted;

            Closing += (s, e) => SSEWVMI.WebEngine.Dispose();
            Loaded += (s, e) => SSEEH.WindowLoaded(this);

            SSEHV.Start();
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            Dispose();

            SSEHR.Control();

            SSEWVHU.SetVolume(SSEHD.GetVolume());

            if (SMMB.PausePerformance)
            {
                SSEWVHU.Pause();

                SSEMI.PausePerformance = true;
            }
            else if (SSEMI.PausePerformance)
            {
                SSEWVHU.Play();

                SSEMI.PausePerformance = false;
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}