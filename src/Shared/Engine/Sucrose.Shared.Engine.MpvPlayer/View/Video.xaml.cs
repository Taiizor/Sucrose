using Microsoft.Win32;
using System.Windows;
using System.Windows.Media;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSEEH = Sucrose.Shared.Engine.Event.Handler;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHR = Sucrose.Shared.Engine.Helper.Run;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEHV = Sucrose.Shared.Engine.Helper.Volume;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEMPHV = Sucrose.Shared.Engine.MpvPlayer.Helper.Video;
using SSEMPMI = Sucrose.Shared.Engine.MpvPlayer.Manage.Internal;

namespace Sucrose.Shared.Engine.MpvPlayer.View
{
    /// <summary>
    /// Interaction logic for Video.xaml
    /// </summary>
    public sealed partial class Video : Window, IDisposable
    {
        public Video(string Video)
        {
            InitializeComponent();

            SystemEvents.DisplaySettingsChanged += (s, e) => SSEEH.DisplaySettingsChanged(this);

            SSEMPMI.Source = SSEHS.GetSource(Video).ToString();

            ContentRendered += (s, e) => SSEEH.ContentRendered(this);

            SSEMPMI.MediaEngine = new(PlayerHost.Handle, SSEMPMI.MediaPath)
            {
                AutoPlay = true,
                Loop = SSEHD.GetLoop(),
                Volume = SSEHD.GetVolume()
            };

            SSEMPMI.MediaEngine.Load(SSEMPMI.Source);

            SSEMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SSEMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SSEMI.GeneralTimer.Start();

            Closing += (s, e) => SSEMPMI.MediaEngine.Dispose();
            Loaded += (s, e) => SSEEH.WindowLoaded(this);

            SSEHV.Start();
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            Dispose();

            SSEHR.Control();

            SSEMPHV.SetLoop(SSEHD.GetLoop());

            SSEMPHV.SetVolume(SSEHD.GetVolume());

            SSEMPHV.SetStretch((Stretch)SSEHD.GetStretch());

            if (SMMM.PausePerformance)
            {
                SSEMPHV.Pause();

                SSEMI.PausePerformance = true;
            }
            else if (SSEMI.PausePerformance)
            {
                SSEMPHV.Play();

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