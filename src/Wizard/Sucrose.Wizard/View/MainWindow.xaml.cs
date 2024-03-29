using Downloader;
using System.ComponentModel;
using System.IO;
using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Input;
using SHV = Skylark.Helper.Versionly;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SSCEUT = Sucrose.Shared.Core.Enum.UpdateType;
using SSCHA = Sucrose.Shared.Core.Helper.Architecture;
using SSCHF = Sucrose.Shared.Core.Helper.Framework;
using SSCHU = Sucrose.Shared.Core.Helper.Update;
using SSDECT = Sucrose.Shared.Dependency.Enum.CompatibilityType;
using SSHG = Skylark.Standard.Helper.GitHub;
using SSIIA = Skylark.Standard.Interface.IAssets;
using SSIIR = Skylark.Standard.Interface.IReleases;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSHS = Sucrose.Shared.Space.Helper.Security;
using SSSZEZ = Sucrose.Shared.SevenZip.Extension.Zip;
using SSSZHZ = Sucrose.Shared.SevenZip.Helper.Zip;
using SWHWI = Skylark.Wing.Helper.WindowInterop;
using SWMI = Sucrose.Wizard.Manage.Internal;
using SWMM = Sucrose.Wizard.Manage.Manager;
using SWNM = Skylark.Wing.Native.Methods;

namespace Sucrose.Wizard.View
{
    /// <summary>
    /// MainWindow.xaml etkileşim mantığı
    /// </summary>
    public partial class MainWindow : Window
    {
        private static List<SSIIR> Releases { get; set; } = new();

        private static string Bundle { get; set; } = string.Empty;

        private static bool HasBundle { get; set; } = false;

        private static SSIIR Release { get; set; } = null;

        private static bool HasFile { get; set; } = false;

        private static int MaxDelay => 5000;

        private static int MinDelay => 1000;

        public MainWindow()
        {
            InitializeComponent();
        }

        private void WindowCorner()
        {
            try
            {
                SWNM.DWMWINDOWATTRIBUTE Attribute = SWNM.DWMWINDOWATTRIBUTE.WindowCornerPreference;
                SWNM.DWM_WINDOW_CORNER_PREFERENCE Preference = SWNM.DWM_WINDOW_CORNER_PREFERENCE.DWMWCP_ROUND;

                SWNM.DwmSetWindowAttribute(SWHWI.Handle(this), Attribute, ref Preference, (uint)Marshal.SizeOf(typeof(uint)));
            }
            catch
            {
                //
            }
        }

        private async Task Start()
        {
            bool State = true;

            await Task.Delay(MinDelay);

            if (StepCache())
            {
                await Task.Delay(MinDelay);

                if (StepNetwork())
                {
                    await Task.Delay(MinDelay);

                    if (StepReleases())
                    {
                        await Task.Delay(MinDelay);

                        if (StepRelease())
                        {
                            await Task.Delay(MinDelay);

                            if (StepSearching())
                            {
                                await Task.Delay(MinDelay);

                                if (await StepDownloading())
                                {
                                    State = false;
                                }
                            }
                        }
                    }
                }
            }

            if (State)
            {
                await Task.Delay(MaxDelay);

                Ring.Visibility = Visibility.Hidden;
                Message.Visibility = Visibility.Hidden;
                Reload.Visibility = Visibility.Visible;
                Progress.Visibility = Visibility.Hidden;
            }
        }

        private bool StepCache()
        {
            try
            {
                Message.Text = "CHECKING TEMPORARY LOCATIONS";

                if (Directory.Exists(SWMM.CachePath))
                {
                    string[] Files = Directory.GetFiles(SWMM.CachePath);

                    foreach (string Record in Files)
                    {
                        File.Delete(Record);
                    }
                }
                else
                {
                    Directory.CreateDirectory(SWMM.CachePath);
                }

                return true;
            }
            catch
            {
                Message.Text = "UNABLE TO CHECK TEMPORARY LOCATIONS";

                return false;
            }
        }

        private bool StepNetwork()
        {
            try
            {
                Message.Text = "CHECKING INTERNET CONNECTION";

                if (SSSHN.GetHostEntry())
                {
                    SSSHS.Apply();

                    return true;
                }
                else
                {
                    Message.Text = "NO INTERNET CONNECTION";

                    return false;
                }
            }
            catch
            {
                Message.Text = "UNABLE TO CHECK INTERNET CONNECTION";

                return false;
            }
        }

        private bool StepRelease()
        {
            try
            {
                Message.Text = "LIST ARE BEING FILTERED";

                Release = Releases.FirstOrDefault(Releasing => !Releasing.PreRelease);

                if (Release == null)
                {
                    Message.Text = "LIST COULD NOT BE FILTERED";

                    return false;
                }
                else
                {
                    Version Latest = SHV.Clear(Release.TagName);

                    List<SSIIA> Assets = Release.Assets;

                    return true;
                }
            }
            catch
            {
                Message.Text = "UNABLE TO FILTER LIST";

                return false;
            }
        }

        private bool StepReleases()
        {
            try
            {
                Message.Text = "GETTING NECESSARY LIST";

                Releases = SSHG.ReleasesList(SMR.Owner, SMR.Repository, SMMM.UserAgent, SMMM.Key);

                if (Releases.Any())
                {
                    return true;
                }
                else
                {
                    Message.Text = "NECESSARY LIST ARE EMPTY";

                    return false;
                }
            }
            catch
            {
                Message.Text = "UNABLE TO GET NECESSARY LIST";

                return false;
            }
        }

        private bool StepSearching()
        {
            try
            {
                Message.Text = "SEARCHING FOR REQUIRED FILES";

                Version Latest = SHV.Clear(Release.TagName);

                List<SSIIA> Assets = Release.Assets;

                if (Assets.Any())
                {
                    foreach (SSIIA Asset in Assets)
                    {
                        string Name = $"{SMR.AppName}_{SMR.Bundle}_{SSCHF.GetDescription()}_{SSCHA.Get()}_{Latest}{SSCHU.GetDescription(SSCEUT.Compressed)}";

                        string[] Required =
                        {
                            SSCHU.GetDescription(SSCEUT.Compressed),
                            SSCHF.GetDescription(),
                            SSCHA.GetText(),
                            $"{Latest}",
                            SMR.AppName,
                            SMR.Bundle
                        };

                        if (Asset.Name.Contains(Name) && Required.All(Asset.Name.Contains))
                        {
                            SWMI.Source = Asset.BrowserDownloadUrl;

                            Bundle = Path.Combine(SWMM.CachePath, Path.GetFileName(SWMI.Source));

                            if (File.Exists(Bundle))
                            {
                                File.Delete(Bundle);
                            }

                            return true;
                        }
                    }

                    return false;
                }
                else
                {
                    Message.Text = "REQUIRED FILES NOT FOUND";

                    return false;
                }
            }
            catch
            {
                Message.Text = "UNABLE TO SEARCH FOR REQUIRED FILES";

                return false;
            }
        }

        private async Task StepExtracting()
        {
            try
            {
                Message.Text = "EXTRACTING THE DOWNLOADED FILE";

                Ring.Visibility = Visibility.Hidden;
                Message.Visibility = Visibility.Visible;
                Progress.Visibility = Visibility.Hidden;

                if (HasBundle)
                {
                    if (await Task.Run(() => SSSZHZ.CheckArchive(Bundle)))
                    {
                        SSDECT Result = await Task.Run(() => SSSZEZ.Extract(Bundle, SWMM.CachePath));

                        if (Result == SSDECT.Pass)
                        {
                            await Task.Delay(MinDelay);

                            Bundle = Path.ChangeExtension(Bundle, SSCHU.GetDescription(SSCEUT.Executable));

                            Message.Text = "THE EXTRACTED FILE IS BEING EXECUTED";

                            await Task.Delay(MinDelay);

                            await Task.Run(() => SSSHP.Run(Bundle));

                            Message.Text = "THE EXTRACTED FILE HAS BEEN EXECUTED";
                        }
                        else
                        {
                            Message.Text = "UNABLE TO EXTRACT THE DOWNLOADED FILE";
                        }
                    }
                    else
                    {
                        Message.Text = "THE DOWNLOADED FILE IS NOT A VALID ARCHIVE";
                    }
                }
                else
                {
                    Message.Text = "THE DOWNLOADED FILE IS NOT AVAILABLE";
                }
            }
            catch
            {
                Message.Text = "THE DOWNLOADED FILE COULD NOT BE EXTRACTED";
            }

            await Task.Delay(MaxDelay);

            Ring.Visibility = Visibility.Hidden;
            Message.Visibility = Visibility.Hidden;
            Reload.Visibility = Visibility.Visible;
            Progress.Visibility = Visibility.Hidden;
        }

        private async Task<bool> StepDownloading()
        {
            try
            {
                Message.Text = "DOWNLOADING FOR REQUIRED FILE";

                SWMI.DownloadService = new(SWMI.DownloadConfiguration);

                SWMI.DownloadService.DownloadStarted += OnDownloadStarted;
                SWMI.DownloadService.DownloadFileCompleted += OnDownloadFileCompleted;
                SWMI.DownloadService.DownloadProgressChanged += OnDownloadProgressChanged;

                await SWMI.DownloadService.DownloadFileTaskAsync(SWMI.Source, Bundle);

                return true;
            }
            catch
            {
                Message.Text = "UNABLE TO DOWNLOAD FOR REQUIRED FILE";

                return false;
            }
        }

        private async Task Reloader()
        {
            HasFile = false;
            HasBundle = false;

            Bundle = string.Empty;

            Release = null;
            Releases = new();

            Message.Text = "PREPARING NECESSARY PREPARATIONS";

            Ring.Progress = 0;
            Progress.Value = 0;

            Ring.Visibility = Visibility.Hidden;
            Reload.Visibility = Visibility.Hidden;
            Message.Visibility = Visibility.Visible;
            Progress.Visibility = Visibility.Hidden;

            await Start();
        }

        private async void Reload_Click(object sender, RoutedEventArgs e)
        {
            Dispose();

            await Reloader();
        }

        private void Window_MouseDown(object sender, MouseButtonEventArgs e)
        {
            if (e.ChangedButton == MouseButton.Left)
            {
                Cursor = Cursors.SizeAll;
                DragMove();
                Cursor = Cursors.Arrow;
            }
        }

        private async void Window_ContentRendered(object sender, EventArgs e)
        {
            WindowCorner();

            await Start();
        }

        private async void OnDownloadStarted(object sender, DownloadStartedEventArgs e)
        {
            await Application.Current.Dispatcher.InvokeAsync(() =>
            {
                HasBundle = true;

                Reload.Visibility = Visibility.Hidden;
                Message.Visibility = Visibility.Hidden;

                if (SWMI.Chance)
                {
                    Ring.Visibility = Visibility.Visible;
                    Progress.Visibility = Visibility.Hidden;
                }
                else
                {
                    Ring.Visibility = Visibility.Hidden;
                    Progress.Visibility = Visibility.Visible;
                }
            });
        }

        private async void OnDownloadFileCompleted(object sender, AsyncCompletedEventArgs e)
        {
            await Application.Current.Dispatcher.InvokeAsync(async () =>
            {
                if (e.Error != null || e.Cancelled)
                {
                    if (e.Error != null)
                    {
                        HasBundle = false;
                        HasFile = true;

                        Message.Text = "AN ERROR OCCURRED WHILE DOWNLOADING THE REQUIRED FILE";
                    }
                    else
                    {
                        HasBundle = false;
                        HasFile = false;

                        Message.Text = "THE REQUIRED FILE DOWNLOAD WAS CANCELED";
                    }

                    Ring.Visibility = Visibility.Hidden;
                    Message.Visibility = Visibility.Visible;
                    Progress.Visibility = Visibility.Hidden;

                    await Task.Delay(MaxDelay);

                    Message.Visibility = Visibility.Hidden;
                    Reload.Visibility = Visibility.Visible;
                }
                else
                {
                    Ring.Progress = 100;
                    Progress.Value = 100;

                    await Task.Delay(MinDelay);

                    await StepExtracting();
                }
            });
        }

        private async void OnDownloadProgressChanged(object sender, DownloadProgressChangedEventArgs e)
        {
            await Application.Current.Dispatcher.InvokeAsync(() =>
            {
                Ring.Progress = e.ProgressPercentage;
                Progress.Value = e.ProgressPercentage;
            });
        }

        private void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}