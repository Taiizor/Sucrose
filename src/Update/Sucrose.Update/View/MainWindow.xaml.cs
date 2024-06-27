using Downloader;
using Newtonsoft.Json;
using System.ComponentModel;
using System.IO;
using System.Net.Http;
using System.Runtime.InteropServices;
using System.Text;
using System.Windows;
using System.Windows.Input;
using System.Windows.Media.Imaging;
using SECNT = Skylark.Enum.ClearNumericType;
using SEMST = Skylark.Enum.ModeStorageType;
using SEST = Skylark.Enum.StorageType;
using SEVT = Skylark.Enum.VersionType;
using SHN = Skylark.Helper.Numeric;
using SHV = Skylark.Helper.Versionly;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SRER = Sucrose.Resources.Extension.Resources;
using SSCECT = Sucrose.Shared.Core.Enum.ChannelType;
using SSCEUT = Sucrose.Shared.Core.Enum.UpdateType;
using SSCHA = Sucrose.Shared.Core.Helper.Architecture;
using SSCHF = Sucrose.Shared.Core.Helper.Framework;
using SSCHU = Sucrose.Shared.Core.Helper.Update;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SSCMM = Sucrose.Shared.Core.Manage.Manager;
using SSDECT = Sucrose.Shared.Dependency.Enum.CompatibilityType;
using SSESSE = Skylark.Standard.Extension.Storage.StorageExtension;
using SSHG = Skylark.Standard.Helper.GitHub;
using SSIIA = Skylark.Standard.Interface.IAssets;
using SSIIR = Skylark.Standard.Interface.IReleases;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSHS = Sucrose.Shared.Space.Helper.Security;
using SSSHU = Sucrose.Shared.Space.Helper.User;
using SSSMUD = Sucrose.Shared.Space.Model.UpdateData;
using SSSZEZ = Sucrose.Shared.SevenZip.Extension.Zip;
using SSSZHZ = Sucrose.Shared.SevenZip.Helper.Zip;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SUMI = Sucrose.Update.Manage.Internal;
using SUMM = Sucrose.Update.Manage.Manager;
using SWHWI = Skylark.Wing.Helper.WindowInterop;
using SWNM = Skylark.Wing.Native.Methods;
using Timer = System.Timers.Timer;

namespace Sucrose.Update.View
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : Window
    {
        private static List<SSIIR> Releases { get; set; } = new();

        private static string Bundle { get; set; } = string.Empty;

        private static List<SSIIA> Assets { get; set; } = new();

        private static Version Latest { get; set; } = new();

        private static bool HasBundle { get; set; } = false;

        private static SSIIR Release { get; set; } = null;

        public static bool Updater { get; set; } = true;

        private static int MinDelay => 1000;

        private static int MaxDelay => 5000;

        public MainWindow()
        {
            InitializeComponent();
        }

        private void LoadBackground()
        {
            BitmapImage Back = new();

            Back.BeginInit();

            Back.UriSource = SMR.Randomise.Next(3) switch
            {
                0 => new Uri("pack://application:,,,/Assets/Back1.jpg", UriKind.RelativeOrAbsolute),
                1 => new Uri("pack://application:,,,/Assets/Back2.jpg", UriKind.RelativeOrAbsolute),
                _ => new Uri("pack://application:,,,/Assets/Back3.jpg", UriKind.RelativeOrAbsolute),
            };

            Back.EndInit();

            Background.Source = Back;
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

                if (await StepNetwork())
                {
                    await Task.Delay(MinDelay);

                    if (StepReleases())
                    {
                        await Task.Delay(MinDelay);

                        if (StepRelease())
                        {
                            await Task.Delay(MinDelay);

                            if (StepComparing())
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
                Message.Text = SRER.GetValue("Update", "MessageText", "Temporary");

                if (Directory.Exists(SUMM.CachePath))
                {
                    string[] Files = Directory.GetFiles(SUMM.CachePath);

                    foreach (string Record in Files)
                    {
                        File.Delete(Record);
                    }
                }
                else
                {
                    Directory.CreateDirectory(SUMM.CachePath);
                }

                return true;
            }
            catch
            {
                Message.Text = SRER.GetValue("Update", "MessageText", "Temporary", "Error");

                return false;
            }
        }

        private async Task<bool> StepNetwork()
        {
            try
            {
                Message.Text = SRER.GetValue("Update", "MessageText", "Connection");

                if (SSSHN.GetHostEntry())
                {
                    SSSHS.Apply();

                    SMMI.UpdateSettingManager.SetSetting(SMC.UpdateTime, DateTime.Now);

                    try
                    {
                        if (SMMM.Statistics)
                        {
                            using HttpClient Client = new();

                            HttpResponseMessage Response = new();

                            Client.DefaultRequestHeaders.Add("User-Agent", SMMM.UserAgent);

                            try
                            {
                                SSSMUD UpdateData = new(SSCHV.GetText());

                                StringContent Content = new(JsonConvert.SerializeObject(UpdateData, Formatting.Indented), Encoding.UTF8, "application/json");

                                Response = await Client.PostAsync($"{SMR.SoferityWebsite}/{SMR.SoferityVersion}/{SMR.SoferityReport}/{SMR.SoferityUpdate}/{SSSHU.GetGuid()}", Content);
                            }
                            catch (Exception Exception)
                            {
                                await SSWW.Watch_CatchException(Exception);
                            }
                        }
                    }
                    catch (Exception Exception)
                    {
                        await SSWW.Watch_CatchException(Exception);
                    }

                    return true;
                }
                else
                {
                    Message.Text = SRER.GetValue("Update", "MessageText", "Connection", "None");

                    return false;
                }
            }
            catch
            {
                Message.Text = SRER.GetValue("Update", "MessageText", "Connection", "Error");

                return false;
            }
        }

        private bool StepRelease()
        {
            try
            {
                Message.Text = SRER.GetValue("Update", "MessageText", "Filtering");

                Release = Releases.FirstOrDefault();

                if (SSCMM.ChannelType == SSCECT.Release)
                {
                    Release = Releases.FirstOrDefault(Releasing => !Releasing.PreRelease);
                }

                if (Release == null)
                {
                    Message.Text = SRER.GetValue("Update", "MessageText", "Filtering", "Not");

                    return false;
                }
                else
                {
                    return true;
                }
            }
            catch
            {
                Message.Text = SRER.GetValue("Update", "MessageText", "Filtering", "Error");

                return false;
            }
        }

        private bool StepReleases()
        {
            try
            {
                Message.Text = SRER.GetValue("Update", "MessageText", "Listing");

                Releases = SSHG.ReleasesList(SMR.Owner, SMR.Repository, SMMM.UserAgent, SMMM.Key);

                if (Releases.Any())
                {
                    return true;
                }
                else
                {
                    Message.Text = SRER.GetValue("Update", "MessageText", "Listing", "Empty");

                    return false;
                }
            }
            catch
            {
                Message.Text = SRER.GetValue("Update", "MessageText", "Listing", "Error");

                return false;
            }
        }

        private bool StepComparing()
        {
            try
            {
                Message.Text = SRER.GetValue("Update", "MessageText", "Comparing");

                Version Current = SSCHV.Get();
                Latest = SHV.Clear(Release.TagName);

                if (SHV.Compare(Current, Latest) == SEVT.Latest)
                {
                    Assets = Release.Assets;

                    return true;
                }
                else
                {
                    Message.Text = SRER.GetValue("Update", "MessageText", "Comparing", "Update");

                    return false;
                }
            }
            catch
            {
                Message.Text = SRER.GetValue("Update", "MessageText", "Comparing", "Error");

                return false;
            }
        }

        private bool StepSearching()
        {
            try
            {
                Message.Text = SRER.GetValue("Update", "MessageText", "Searching");

                if (Assets.Any())
                {
                    foreach (SSIIA Asset in Assets)
                    {
                        string Name = $"{SMR.AppName}_{SMR.Bundle}_{SSCHF.GetDescription()}_{SSCHA.Get()}_{Latest}{SSCHU.GetDescription(SUMI.UpdateType)}";

                        string[] Required =
                        {
                            SSCHU.GetDescription(SUMI.UpdateType),
                            SSCHF.GetDescription(),
                            SSCHA.GetText(),
                            $"{Latest}",
                            SMR.AppName,
                            SMR.Bundle
                        };

                        if (Asset.Name.Contains(Name) && Required.All(Asset.Name.Contains))
                        {
                            SUMI.Source = Asset.BrowserDownloadUrl;

                            Bundle = Path.Combine(SUMM.CachePath, Path.GetFileName(SUMI.Source));

                            if (File.Exists(Bundle))
                            {
                                File.Delete(Bundle);
                            }

                            return true;
                        }
                    }

                    if (string.IsNullOrEmpty(Bundle))
                    {
                        Message.Text = SRER.GetValue("Update", "MessageText", "Searching", "Condition");
                    }

                    return false;
                }
                else
                {
                    Message.Text = SRER.GetValue("Update", "MessageText", "Searching", "Not");

                    return false;
                }
            }
            catch
            {
                Message.Text = SRER.GetValue("Update", "MessageText", "Searching", "Error");

                return false;
            }
        }

        private async Task StepRunning()
        {
            try
            {
                Message.Text = SRER.GetValue("Update", "MessageText", "Running");

                Ring.Visibility = Visibility.Hidden;
                Message.Visibility = Visibility.Visible;
                Progress.Visibility = Visibility.Hidden;

                if (HasBundle)
                {
                    await Task.Delay(MinDelay);

                    Message.Text = SRER.GetValue("Update", "MessageText", "Running", "Executing");

                    await Task.Delay(MinDelay);

                    await Task.Run(() => SSSHP.Run(Bundle));

                    Message.Text = SRER.GetValue("Update", "MessageText", "Running", "Executed");
                }
                else
                {
                    Message.Text = SRER.GetValue("Update", "MessageText", "Running", "Not");
                }
            }
            catch
            {
                Message.Text = SRER.GetValue("Update", "MessageText", "Running", "Error");
            }

            await Task.Delay(MaxDelay);

            Ring.Visibility = Visibility.Hidden;
            Message.Visibility = Visibility.Hidden;
            Reload.Visibility = Visibility.Visible;
            Progress.Visibility = Visibility.Hidden;
        }

        private async Task StepExtracting()
        {
            try
            {
                Message.Text = SRER.GetValue("Update", "MessageText", "Extracting");

                Ring.Visibility = Visibility.Hidden;
                Message.Visibility = Visibility.Visible;
                Progress.Visibility = Visibility.Hidden;

                if (HasBundle)
                {
                    if (await Task.Run(() => SSSZHZ.CheckArchive(Bundle)))
                    {
                        SSDECT Result = await Task.Run(() => SSSZEZ.Extract(Bundle, SUMM.CachePath));

                        if (Result == SSDECT.Pass)
                        {
                            await Task.Delay(MinDelay);

                            Bundle = Path.ChangeExtension(Bundle, SSCHU.GetDescription(SSCEUT.Executable));

                            Message.Text = SRER.GetValue("Update", "MessageText", "Extracting", "Executing");

                            await Task.Delay(MinDelay);

                            await Task.Run(() => SSSHP.Run(Bundle));

                            Message.Text = SRER.GetValue("Update", "MessageText", "Extracting", "Executed");
                        }
                        else
                        {
                            Message.Text = SRER.GetValue("Update", "MessageText", "Extracting", "Extract");
                        }
                    }
                    else
                    {
                        Message.Text = SRER.GetValue("Update", "MessageText", "Extracting", "Corrupt");
                    }
                }
                else
                {
                    Message.Text = SRER.GetValue("Update", "MessageText", "Extracting", "Not");
                }
            }
            catch
            {
                Message.Text = SRER.GetValue("Update", "MessageText", "Extracting", "Error");
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
                Message.Text = SRER.GetValue("Update", "MessageText", "Downloading");

                UpdateLimit();

                SUMI.DownloadService = new(SUMI.DownloadConfiguration);

                SUMI.DownloadService.DownloadStarted += OnDownloadStarted;
                SUMI.DownloadService.DownloadFileCompleted += OnDownloadFileCompleted;
                SUMI.DownloadService.DownloadProgressChanged += OnDownloadProgressChanged;

                await SUMI.DownloadService.DownloadFileTaskAsync(SUMI.Source, Bundle);

                return true;
            }
            catch
            {
                Message.Text = SRER.GetValue("Update", "MessageText", "Downloading", "Error");

                return false;
            }
        }

        private async Task Reloader()
        {
            HasBundle = false;

            Bundle = string.Empty;

            Release = null;
            Releases = new();

            Message.Text = SRER.GetValue("Update", "MessageText", "Preparing");

            Ring.Progress = 0;
            Progress.Value = 0;

            Ring.Visibility = Visibility.Hidden;
            Reload.Visibility = Visibility.Hidden;
            Message.Visibility = Visibility.Visible;
            Progress.Visibility = Visibility.Hidden;

            await Start();
        }

        private static void UpdateLimit()
        {
            if (Updater)
            {
                Updater = false;

                Timer Limiter = new(3000);

                Limiter.Elapsed += (s, e) =>
                {
                    try
                    {
                        if (SMMM.UpdateLimitValue > 0)
                        {
                            double UpdateLimit = SSESSE.Convert(SMMM.UpdateLimitValue, SMMM.UpdateLimitType, SEST.Byte, SEMST.Palila);

                            long Limit = Convert.ToInt64(SHN.Numeral(UpdateLimit, false, false, 0, '0', SECNT.None));

                            SUMI.DownloadConfiguration.MaximumBytesPerSecond = Limit;
                        }
                        else
                        {
                            SUMI.DownloadConfiguration.MaximumBytesPerSecond = 0;
                        }
                    }
                    catch
                    {
                        SUMI.DownloadConfiguration.MaximumBytesPerSecond = 0;
                    }
                };

                Limiter.AutoReset = true;

                Limiter.Start();
            }
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
            LoadBackground();

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

                if (SUMI.Chance)
                {
                    Ring.Visibility = Visibility.Visible;
                    Progress.Visibility = Visibility.Hidden;
                }
                else
                {
                    Ring.Visibility = Visibility.Hidden;
                    Progress.Visibility = Visibility.Visible;
                }

                SMMI.UpdateSettingManager.SetSetting(SMC.UpdateState, true);
            });
        }

        private async void OnDownloadFileCompleted(object sender, AsyncCompletedEventArgs e)
        {
            await Application.Current.Dispatcher.InvokeAsync(async () =>
            {
                if (e.Error != null || e.Cancelled)
                {
                    HasBundle = false;

                    if (e.Error != null)
                    {
                        Message.Text = SRER.GetValue("Update", "MessageText", "Downloading", "Complete", "Error");
                    }
                    else
                    {
                        Message.Text = SRER.GetValue("Update", "MessageText", "Downloading", "Complete", "Cancel");
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

                    switch (SUMI.UpdateType)
                    {
                        case SSCEUT.Compressed:
                            await StepExtracting();
                            break;
                        case SSCEUT.Executable:
                            await StepRunning();
                            break;
                        default:
                            break;
                    }
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