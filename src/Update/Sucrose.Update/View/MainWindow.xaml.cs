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
using Wpf.Ui.TaskBar;
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
using SSCEUCT = Sucrose.Shared.Core.Enum.UpdateChannelType;
using SSCEUET = Sucrose.Shared.Core.Enum.UpdateExtensionType;
using SSCHA = Sucrose.Shared.Core.Helper.Architecture;
using SSCHF = Sucrose.Shared.Core.Helper.Framework;
using SSCHU = Sucrose.Shared.Core.Helper.Update;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SSCMM = Sucrose.Shared.Core.Manage.Manager;
using SSDECDT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSDECYT = Sucrose.Shared.Dependency.Enum.CompatibilityType;
using SSDEUMT = Sucrose.Shared.Dependency.Enum.UpdateModuleType;
using SSDEUST = Sucrose.Shared.Dependency.Enum.UpdateServerType;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SSESSE = Skylark.Standard.Extension.Storage.StorageExtension;
using SSHG = Skylark.Standard.Helper.GitHub;
using SSIIA = Skylark.Standard.Interface.IAssets;
using SSIIR = Skylark.Standard.Interface.IReleases;
using SSSEPS = Sucrose.Shared.Space.Extension.ProgressStream;
using SSSHE = Sucrose.Shared.Space.Helper.Extension;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSHS = Sucrose.Shared.Space.Helper.Security;
using SSSHU = Sucrose.Shared.Space.Helper.User;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SSSMUD = Sucrose.Shared.Space.Model.UpdateData;
using SSSZEZ = Sucrose.Shared.SevenZip.Extension.Zip;
using SSSZHZ = Sucrose.Shared.SevenZip.Helper.Zip;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SUHU = Sucrose.Update.Helper.Update;
using SUMI = Sucrose.Update.Manage.Internal;
using SUMM = Sucrose.Update.Manage.Manager;
using SWHWI = Skylark.Wing.Helper.WindowInterop;
using SWNM = Skylark.Wing.Native.Methods;

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

        private static double Value { get; set; } = 0;

        private static int MaxCount { get; set; } = 5;

        private static int Count { get; set; } = 0;

        private static int MinDelay => 1000;

        private static int MaxDelay => 5000;

        private bool Silent;

        public MainWindow(bool Silent)
        {
            this.Silent = Silent;
            InitializeComponent();
        }

        private void LoadBackground()
        {
            BitmapImage Back = new();

            Back.BeginInit();

            Back.UriSource = SMR.Randomise.Next(5) switch
            {
                0 => new Uri("pack://application:,,,/Assets/Back1.jpg", UriKind.RelativeOrAbsolute),
                1 => new Uri("pack://application:,,,/Assets/Back2.jpg", UriKind.RelativeOrAbsolute),
                2 => new Uri("pack://application:,,,/Assets/Back3.jpg", UriKind.RelativeOrAbsolute),
                3 => new Uri("pack://application:,,,/Assets/Back4.jpg", UriKind.RelativeOrAbsolute),
                _ => new Uri("pack://application:,,,/Assets/Back5.jpg", UriKind.RelativeOrAbsolute),
            };

            Back.EndInit();

            Background.Source = Back;
        }

        private void WindowCorner()
        {
            try
            {
                if (!Silent)
                {
                    SWNM.DWMWINDOWATTRIBUTE Attribute = SWNM.DWMWINDOWATTRIBUTE.WindowCornerPreference;
                    SWNM.DWM_WINDOW_CORNER_PREFERENCE Preference = SWNM.DWM_WINDOW_CORNER_PREFERENCE.DWMWCP_ROUND;

                    SWNM.DwmSetWindowAttribute(SWHWI.Handle(this), Attribute, ref Preference, (uint)Marshal.SizeOf(typeof(uint)));
                }
            }
            catch { }
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

                                    if (!Silent || StepSilent())
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
            }

            if (Silent)
            {
                Close();
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

        private bool StepSilent()
        {
            try
            {
                Show();

                Opacity = 1;

                Silent = false;

                WindowCorner();

                ShowInTaskbar = true;

                Visibility = Visibility.Visible;

                return true;
            }
            catch
            {
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
                                SSSMUD UpdateData = new(Silent, SSCHV.GetText());

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

                if (SSCMM.UpdateChannelType == SSCEUCT.Release)
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

                Releases = SSDMM.UpdateServerType switch
                {
                    SSDEUST.GitHub => SSHG.ReleasesList(SMR.Owner, SMR.Repository, SMMM.UserAgent, SMMM.Key),
                    SSDEUST.Soferity => SUHU.ReleasesList($"{SMR.SoferityWebsite}/{SMR.SoferityUpdate}", SMMM.UserAgent),
                    _ => new(),
                };

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

                    Reload.Content = SRER.GetValue("Update", "ReloadText", "Check", "Again");

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
                        string Name = $"{SMR.AppName}_{SMR.Bundle}_{SSCHF.GetDescription()}_{SSCHA.Get()}_{Latest}{SSCHU.GetDescription(SUMI.UpdateExtensionType)}";

                        string[] Required =
                        {
                            SSCHU.GetDescription(SUMI.UpdateExtensionType),
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

                TaskBarProgress.SetState(this, TaskBarProgressState.None);

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

                TaskBarProgress.SetState(this, TaskBarProgressState.None);

                if (HasBundle)
                {
                    if (await Task.Run(() => SSSZHZ.CheckArchive(Bundle)))
                    {
                        SSDECYT Result = await Task.Run(() => SSSZEZ.Extract(Bundle, SUMM.CachePath));

                        if (Result == SSDECYT.Pass)
                        {
                            await Task.Delay(MinDelay);

                            Bundle = SSSHE.Change(Bundle, SSCHU.GetDescription(SSCEUET.Executable));

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

                CheckProgress();

                switch (SUMI.UpdateModuleType)
                {
                    case SSDEUMT.Native:
                        {
                            int BufferSize = 8192;

                            using HttpClient Client = new();

                            Client.DefaultRequestHeaders.Add("User-Agent", SMMM.UserAgent);

                            using HttpResponseMessage Response = await Client.GetAsync(SUMI.Source, HttpCompletionOption.ResponseHeadersRead);

                            Response.EnsureSuccessStatusCode();

                            long TotalBytes = Response.Content.Headers.ContentLength ?? -1L;

                            using Stream ContentStream = await Response.Content.ReadAsStreamAsync();
                            using FileStream FileStream = new(Bundle, FileMode.OpenOrCreate, FileAccess.Write, FileShare.None, BufferSize, true);
                            using SSSEPS ProgressStream = new(ContentStream, TotalBytes, ReportProgress);

                            ProgressStream.ProgressCompleted += ProgressStream_ProgressCompleted;
                            ProgressStream.ProgressStarted += ProgressStream_ProgressStarted;
                            ProgressStream.ProgressFailed += ProgressStream_ProgressFailed;

                            byte[] Buffer = new byte[BufferSize];
                            long BytesDownloadedThisSecond = 0;
                            DateTime StartTime = DateTime.Now;
                            int BytesRead;

#if NET48_OR_GREATER
                            while ((BytesRead = await ProgressStream.ReadAsync(Buffer, 0, Buffer.Length)) > 0)
#else
                            while ((BytesRead = await ProgressStream.ReadAsync(Buffer)) > 0)
#endif
                            {
#if NET48_OR_GREATER
                                await FileStream.WriteAsync(Buffer, 0, BytesRead);
#else
                                await FileStream.WriteAsync(Buffer.AsMemory(0, BytesRead));
#endif

                                long Limit = GetLimit();

                                if (Limit > 0)
                                {
                                    BytesDownloadedThisSecond += BytesRead;

                                    if (BytesDownloadedThisSecond >= Limit)
                                    {
                                        TimeSpan Elapsed = DateTime.Now - StartTime;

                                        if (Elapsed.TotalSeconds < 1)
                                        {
                                            await Task.Delay(TimeSpan.FromSeconds(1) - Elapsed);
                                        }

                                        BytesDownloadedThisSecond = 0;

                                        StartTime = DateTime.Now;
                                    }
                                }
                            }
                        }

                        return true;
                    case SSDEUMT.Downloader:
                        UpdateLimit();

                        SUMI.DownloadService = new(SUMI.DownloadConfiguration);

                        SUMI.DownloadService.DownloadStarted += OnDownloadStarted;
                        SUMI.DownloadService.DownloadFileCompleted += OnDownloadFileCompleted;
                        SUMI.DownloadService.DownloadProgressChanged += OnDownloadProgressChanged;

                        await SUMI.DownloadService.DownloadFileTaskAsync(SUMI.Source, Bundle);

                        return true;
                    default:
                        return true;
                }
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

            Count = 0;
            Value = 0;

            Ring.Progress = 0;
            Progress.Value = 0;

            Ring.Visibility = Visibility.Hidden;
            Reload.Visibility = Visibility.Hidden;
            Message.Visibility = Visibility.Visible;
            Progress.Visibility = Visibility.Hidden;

            Reload.Content = SRER.GetValue("Update", "ReloadText");

            await Start();
        }

        private static long GetLimit()
        {
            try
            {
                if (SMMM.UpdateLimitValue > 0)
                {
                    double UpdateLimit = SSESSE.Convert(SMMM.UpdateLimitValue, SMMM.UpdateLimitType, SEST.Byte, SEMST.Palila);

                    long Limit = Convert.ToInt64(SHN.Numeral(UpdateLimit, false, false, 0, '0', SECNT.None));

                    return Limit;
                }
                else
                {
                    return 0;
                }
            }
            catch
            {
                return 0;
            }
        }

        private static void UpdateLimit()
        {
            if (!SUMI.Limiter.Enabled)
            {
                SUMI.Limiter.Elapsed += (s, e) =>
                {
                    SUMI.DownloadConfiguration.MaximumBytesPerSecond = GetLimit();
                };

                SUMI.Limiter.Start();
            }
        }

        private async void CheckProgress()
        {
            if (!SUMI.Checker.Enabled)
            {
                SUMI.Checker.Elapsed += async (s, e) =>
                {
                    await Application.Current.Dispatcher.InvokeAsync(async () =>
                    {
                        if (Ring.Progress > 0 && Progress.Value < 100)
                        {
                            if (Value != Progress.Value)
                            {
                                Value = Ring.Progress;

                                Count = 0;
                            }
                            else if (Value != 0)
                            {
                                Count++;

                                if (Count > MaxCount)
                                {
                                    Count = 0;
                                    Value = 0;

                                    await SUMI.DownloadService.CancelTaskAsync();
                                }
                            }
                        }
                        else
                        {
                            Count = 0;
                        }
                    });
                };

                SUMI.Checker.Start();
            }

            await Task.CompletedTask;
        }

        private async void Reload_Click(object sender, RoutedEventArgs e)
        {
            Dispose();

            if (SUMI.Trying && !string.IsNullOrWhiteSpace(SUMI.Source))
            {
                SUMI.Trying = false;

                Reload.IsEnabled = false;

                Reload.Content = SRER.GetValue("Update", "ReloadText", "Browser", "Opening");

                await Task.Delay(MinDelay);

                if (SUMI.UpdateExtensionType == SSCEUET.Compressed)
                {
                    SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECDT.Bundle}{SMR.ValueSeparator}{SSSHE.Change(SUMI.Source, SSCHU.GetDescription(SSCEUET.Executable))}");
                }
                else
                {
                    SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECDT.Bundle}{SMR.ValueSeparator}{SUMI.Source}");
                }

                Reload.Content = SRER.GetValue("Update", "ReloadText", "Browser", "Opened");

                await Task.Delay(MaxDelay);

                Reload.IsEnabled = true;

                Reload.Content = SRER.GetValue("Update", "ReloadText");
            }
            else
            {
                await Reloader();
            }
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

        private async void ProgressStream_ProgressFailed(object sender, Exception e)
        {
            await Application.Current.Dispatcher.InvokeAsync(async () =>
            {
                Count = 0;
                Value = 0;

                HasBundle = false;

                SUMI.Trying = true;

                Message.Text = SRER.GetValue("Update", "MessageText", "Downloading", "Complete", "Error");

                Ring.Visibility = Visibility.Hidden;
                Message.Visibility = Visibility.Visible;
                Progress.Visibility = Visibility.Hidden;

                TaskBarProgress.SetState(this, TaskBarProgressState.Error);

                await Task.Delay(MaxDelay);

                Message.Visibility = Visibility.Hidden;
                Reload.Visibility = Visibility.Visible;

                TaskBarProgress.SetState(this, TaskBarProgressState.None);
            });
        }

        private async void ProgressStream_ProgressStarted(object sender, EventArgs e)
        {
            await Application.Current.Dispatcher.InvokeAsync(() =>
            {
                Count = 0;
                Value = 0;

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

                TaskBarProgress.SetValue(this, TaskBarProgressState.Normal, 0);

                SMMI.UpdateSettingManager.SetSetting(SMC.UpdateState, true);
            });
        }

        private async void OnDownloadStarted(object sender, DownloadStartedEventArgs e)
        {
            await Application.Current.Dispatcher.InvokeAsync(() =>
            {
                Count = 0;
                Value = 0;

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

                TaskBarProgress.SetValue(this, TaskBarProgressState.Normal, 0);

                SMMI.UpdateSettingManager.SetSetting(SMC.UpdateState, true);
            });
        }

        private async void ProgressStream_ProgressCompleted(object sender, EventArgs e)
        {
            await Application.Current.Dispatcher.InvokeAsync(async () =>
            {
                Count = 0;
                Value = 100;

                Ring.Progress = 100;
                Progress.Value = 100;

                TaskBarProgress.SetValue(this, TaskBarProgressState.Normal, 100);

                await Task.Delay(MinDelay);

                switch (SUMI.UpdateExtensionType)
                {
                    case SSCEUET.Compressed:
                        await StepExtracting();
                        break;
                    case SSCEUET.Executable:
                        await StepRunning();
                        break;
                    default:
                        break;
                }
            });
        }

        private async void OnDownloadFileCompleted(object sender, AsyncCompletedEventArgs e)
        {
            await Application.Current.Dispatcher.InvokeAsync(async () =>
            {
                if (e.Error != null || e.Cancelled)
                {
                    Count = 0;
                    Value = 0;

                    HasBundle = false;

                    SUMI.Trying = true;

                    if (e.Cancelled)
                    {
                        Message.Text = SRER.GetValue("Update", "MessageText", "Downloading", "Complete", "Cancel");
                    }
                    else
                    {
                        Message.Text = SRER.GetValue("Update", "MessageText", "Downloading", "Complete", "Error");
                    }

                    Ring.Visibility = Visibility.Hidden;
                    Message.Visibility = Visibility.Visible;
                    Progress.Visibility = Visibility.Hidden;

                    TaskBarProgress.SetState(this, TaskBarProgressState.Error);

                    await Task.Delay(MaxDelay);

                    Message.Visibility = Visibility.Hidden;
                    Reload.Visibility = Visibility.Visible;

                    TaskBarProgress.SetState(this, TaskBarProgressState.None);
                }
                else
                {
                    Count = 0;
                    Value = 100;

                    Ring.Progress = 100;
                    Progress.Value = 100;

                    TaskBarProgress.SetValue(this, TaskBarProgressState.Normal, 100);

                    await Task.Delay(MinDelay);

                    switch (SUMI.UpdateExtensionType)
                    {
                        case SSCEUET.Compressed:
                            await StepExtracting();
                            break;
                        case SSCEUET.Executable:
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

                TaskBarProgress.SetValue(this, TaskBarProgressState.Normal, Convert.ToInt32(e.ProgressPercentage));
            });
        }

        private async void ReportProgress(long bytesTransferred, long totalBytes, double progress)
        {
            await Application.Current.Dispatcher.InvokeAsync(() =>
            {
                Ring.Progress = progress;
                Progress.Value = progress;

                TaskBarProgress.SetValue(this, TaskBarProgressState.Normal, Convert.ToInt32(progress));
            });
        }

        private void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}