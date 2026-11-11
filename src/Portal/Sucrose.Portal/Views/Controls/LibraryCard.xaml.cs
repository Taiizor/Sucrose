using System.ComponentModel;
using System.Diagnostics;
using System.IO;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using Wpf.Ui.Controls;
using MessageBox = Wpf.Ui.Controls.MessageBox;
using SHV = Skylark.Helper.Versionly;
using SMMB = Sucrose.Manager.Manage.Backgroundog;
using SMMC = Sucrose.Manager.Manage.Cycling;
using SMMCC = Sucrose.Memory.Manage.Constant.Cycling;
using SMMCL = Sucrose.Memory.Manage.Constant.Library;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMML = Sucrose.Manager.Manage.Library;
using SMMP = Sucrose.Manager.Manage.Portal;
using SMMRC = Sucrose.Memory.Manage.Readonly.Content;
using SMMRF = Sucrose.Memory.Manage.Readonly.Folder;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SPEIL = Sucrose.Portal.Extension.ImageLoader;
using SPVCTD = Sucrose.Portal.Views.Controls.ThemeDelete;
using SPVCTE = Sucrose.Portal.Views.Controls.ThemeEdit;
using SPVCTR = Sucrose.Portal.Views.Controls.ThemeReview;
using SPVCTS = Sucrose.Portal.Views.Controls.ThemeShare;
using SRER = Sucrose.Resources.Extension.Resources;
using SRHR = Sucrose.Resources.Helper.Resources;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSDEET = Sucrose.Shared.Dependency.Enum.EngineType;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSDMME = Sucrose.Shared.Dependency.Manage.Manager.Engine;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSSHF = Sucrose.Shared.Space.Helper.Filing;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSHQ = Sucrose.Shared.Space.Helper.Lockscreen;
using SSSHW = Sucrose.Shared.Space.Helper.WindowsImage;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SSTCLC = Sucrose.Shared.Theme.Converter.LocalizationConverter;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSWEW = Sucrose.Shared.Watchdog.Extension.Watch;
using SXAGAB = Sucrose.XamlAnimatedGif.AnimationBehavior;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// LibraryCard.xaml etkileşim mantığı
    /// </summary>
    public partial class LibraryCard : UserControl, IDisposable
    {
        private readonly string Theme = string.Empty;
        private readonly SPEIL Loader = new();
        private SSTHI Info = new();
        public bool Delete;

        internal LibraryCard(string Theme, SSTHI Info)
        {
            this.Info = Info;
            this.Theme = Theme;

            InitializeComponent();
        }

        private void Use()
        {
            if (Directory.Exists(Theme))
            {
                if ((!SMMB.ClosePerformance && !SMMB.PausePerformance) || !SSSHP.Work(SSSMI.Backgroundog))
                {
                    if (SMML.Selected != Path.GetFileName(Theme) || !SSSHL.Run())
                    {
                        SMMI.LibrarySettingManager.SetSetting(SMMCL.Selected, Path.GetFileName(Theme));

                        if (SSSHL.Run())
                        {
                            SSLHK.Stop();
                        }

                        SSLHR.Start();

                        Cursor = Cursors.Arrow;
                    }
                }
            }
        }

        private void UpdateInfo()
        {
            (ThemeTitle.Text, ThemeDescription.Text) = SSTCLC.Convert(Info);

            ToolTip TitleTip = new()
            {
                Content = ThemeTitle.Text
            };

            ToolTip DescriptionTip = new()
            {
                Content = ThemeDescription.Text
            };

            ThemeTitle.ToolTip = TitleTip;
            ThemeDescription.ToolTip = DescriptionTip;

            ThemeTitle.FlowDirection = SRHR.IsRightToLeftText(ThemeTitle.Text) ? FlowDirection.RightToLeft : FlowDirection.LeftToRight;
            ThemeDescription.FlowDirection = SRHR.IsRightToLeftText(ThemeDescription.Text) ? FlowDirection.RightToLeft : FlowDirection.LeftToRight;
        }

        private void MenuUse_Click(object sender, RoutedEventArgs e)
        {
            if (Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
            {
                Use();
            }
        }

        private void MenuFind_Click(object sender, RoutedEventArgs e)
        {
            if (Directory.Exists(Theme))
            {
                SSSHP.Run(Theme);
            }
        }

        private void MenuPreview_Click(object sender, RoutedEventArgs e)
        {
            if (Directory.Exists(Theme))
            {
                return;
            }
        }

        private void MenuCustomize_Click(object sender, RoutedEventArgs e)
        {
            if (Directory.Exists(Theme))
            {
                SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.PropertyA}{SMMRG.ValueSeparator}{SSSMI.Property}{SMMRG.ValueSeparator}{Path.GetFileName(Theme)}");
            }
        }

        private async void MenuLockScreen_Click(object sender, RoutedEventArgs e)
        {
            if (!Directory.Exists(Theme) || Info.Type is not SSDEWT.Gif and not SSDEWT.Video)
            {
                return;
            }

            MenuLockScreen.IsEnabled = false;

            try
            {
                string Source = Info.Source;

                if (!Path.IsPathRooted(Source))
                {
                    Source = Path.Combine(Theme, Source);
                }

                if (!File.Exists(Source))
                {
                    await ShowLockScreenMessageAsync(
                        SRER.GetValue("Portal", "LibraryCard", "LockScreenErrorTitle"),
                        string.Format(SRER.GetValue("Portal", "LibraryCard", "LockScreenErrorMessage"), Source),
                        null);
                    return;
                }

                // Prerequisite: Windows must be in Picture mode with an existing image cache.
                SSSHQ.LockScreenMode Mode = SSSHQ.TryGetLockScreenMode();

                if (Mode != SSSHQ.LockScreenMode.PictureOrOther)
                {
                    Wpf.Ui.Controls.MessageBoxResult ModeResult = await ShowLockScreenMessageAsync(
                        SRER.GetValue("Portal", "LibraryCard", "LockScreenModeTitle"),
                        SRER.GetValue("Portal", "LibraryCard", "LockScreenModeMessage"),
                        SRER.GetValue("Portal", "LibraryCard", "LockScreenModeContinue"));

                    if (ModeResult != Wpf.Ui.Controls.MessageBoxResult.Primary)
                    {
                        return;
                    }
                }

                try
                {
                    if (SSSHQ.FindCacheFolders(SSSHQ.GetCacheDirectory()).Length == 0)
                    {
                        await ShowLockScreenMessageAsync(
                            SRER.GetValue("Portal", "LibraryCard", "LockScreenErrorTitle"),
                            SRER.GetValue("Portal", "LibraryCard", "LockScreenModeMessage"),
                            null);
                        return;
                    }
                }
                catch (Exception Exception) when (!SSSHQ.IsAccessDenied(Exception))
                {
                    await SSWEW.Watch_CatchException(Exception);

                    await ShowLockScreenMessageAsync(
                        SRER.GetValue("Portal", "LibraryCard", "LockScreenErrorTitle"),
                        string.Format(SRER.GetValue("Portal", "LibraryCard", "LockScreenErrorMessage"), Exception.Message),
                        null);
                    return;
                }
                catch (Exception Exception)
                {
                    // The cache is unreadable without elevation. Do not block here;
                    // Commandog repairs access under elevation and reports the result.
                    await SSWEW.Watch_CatchException(Exception);
                }

                string GifPath = Source;
                string VideoTemporary = null;

                if (Info.Type == SSDEWT.Video)
                {
                    GifPath = await EnsureVideoGifAsync(Source);

                    if (GifPath is null)
                    {
                        return;
                    }

                    VideoTemporary = Path.GetDirectoryName(GifPath);
                }
                else if (!SSSHQ.IsGif(Source))
                {
                    await ShowLockScreenMessageAsync(
                        SRER.GetValue("Portal", "LibraryCard", "LockScreenErrorTitle"),
                        string.Format(SRER.GetValue("Portal", "LibraryCard", "LockScreenErrorMessage"), Source),
                        null);
                    return;
                }

                // With UAC off and an admin token there is no prompt, so run directly.
                // Otherwise the replacement runs elevated through Commandog.
                bool Elevated = SSSHQ.IsAdministrator();
                string ResultFile = Path.Combine(SMMRP.Temp, $"SucroseLockScreen_{Guid.NewGuid():N}.txt");

                if (!Elevated)
                {
                    Wpf.Ui.Controls.MessageBoxResult AdminResult = await ShowLockScreenMessageAsync(
                        SRER.GetValue("Portal", "LibraryCard", "LockScreenAdminTitle"),
                        SRER.GetValue("Portal", "LibraryCard", "LockScreenAdminMessage"),
                        SRER.GetValue("Portal", "LibraryCard", "LockScreenAdminApply"));

                    if (AdminResult != Wpf.Ui.Controls.MessageBoxResult.Primary)
                    {
                        CleanupLockScreenTemporary(ResultFile, VideoTemporary);
                        return;
                    }
                }

                try
                {
                    if (!File.Exists(SSSMI.Commandog))
                    {
                        throw new FileNotFoundException(SSSMI.Commandog);
                    }

                    string Arguments = $"{SMMRG.StartCommand}{SSDECT.LockScreenApply}{SMMRG.ValueSeparator}{GifPath}{SMMRG.ValueSeparator}{ResultFile}";

                    if (Elevated)
                    {
                        SSSHP.Run(SSSMI.Commandog, Arguments);
                    }
                    else
                    {
                        SSSHP.Runas(SSSMI.Commandog, Arguments);
                    }

                    // The launched helper reports through the result file. Poll for
                    // it instead of a process handle, which console targets never get.
                    bool Reported = await WaitLockScreenResultAsync(ResultFile);
                    bool Applied = await ShowLockScreenResultAsync(ResultFile, Reported);

                    if (Applied)
                    {
                        await CheckWindowsImageFeatureAsync();
                    }
                }
                catch (Exception Exception) when (Exception is Win32Exception or OperationCanceledException)
                {
                    await SSWEW.Watch_CatchException(Exception);

                    await ShowLockScreenMessageAsync(
                        SRER.GetValue("Portal", "LibraryCard", "LockScreenErrorTitle"),
                        string.Format(SRER.GetValue("Portal", "LibraryCard", "LockScreenErrorMessage"), Exception.Message),
                        null);
                }
                finally
                {
                    CleanupLockScreenTemporary(ResultFile, VideoTemporary);
                }
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);

                await ShowLockScreenMessageAsync(
                    SRER.GetValue("Portal", "LibraryCard", "LockScreenErrorTitle"),
                    string.Format(SRER.GetValue("Portal", "LibraryCard", "LockScreenErrorMessage"), Exception.Message),
                    null);
            }
            finally
            {
                MenuLockScreen.IsEnabled = true;
            }
        }

        private static async Task<bool> WaitLockScreenResultAsync(string ResultFile)
        {
            for (int Wait = 0; Wait < 180; Wait++)
            {
                await Task.Delay(500);

                try
                {
                    if (File.Exists(ResultFile))
                    {
                        return true;
                    }
                }
                catch
                {
                }
            }

            return false;
        }

        private async Task<bool> ShowLockScreenResultAsync(string ResultFile, bool Reported)
        {
            string Title = SRER.GetValue("Portal", "LibraryCard", "LockScreenDoneTitle");
            string Message = SRER.GetValue("Portal", "LibraryCard", "LockScreenDoneMessage");
            bool Applied = Reported;

            try
            {
                if (Reported && File.Exists(ResultFile))
                {
                    string[] Parts = File.ReadAllText(ResultFile).Split('|');

                    if (Parts.Length >= 3 && Parts[0] == "OK" && int.TryParse(Parts[1], out int Succeeded))
                    {
                        Message = string.Format(SRER.GetValue("Portal", "LibraryCard", "LockScreenAppliedMessage"), Succeeded);
                    }
                    else
                    {
                        Applied = false;

                        Title = SRER.GetValue("Portal", "LibraryCard", "LockScreenErrorTitle");

                        Message = Parts.Length >= 2 && Parts[0] == "ERR"
                            ? string.Format(SRER.GetValue("Portal", "LibraryCard", "LockScreenErrorMessage"), Parts[1])
                            : SRER.GetValue("Portal", "LibraryCard", "LockScreenTimeoutMessage");
                    }
                }
            }
            catch
            {
            }

            if (!Reported)
            {
                Applied = false;
                Title = SRER.GetValue("Portal", "LibraryCard", "LockScreenErrorTitle");
                Message = SRER.GetValue("Portal", "LibraryCard", "LockScreenTimeoutMessage");
            }

            await ShowLockScreenMessageAsync(Title, Message, null);

            return Applied;
        }

        private async Task CheckWindowsImageFeatureAsync()
        {
            if (!SSSHW.IsCheckRequired())
            {
                return;
            }

            SSSHW.FeatureState State;

            try
            {
                State = SSSHW.Read();
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
                return;
            }

            if (SSSHW.IsDisabled(State))
            {
                return;
            }

            if (!SSSHW.IsEnabled(State))
            {
                await ShowLockScreenMessageAsync(
                    SRER.GetValue("Portal", "LibraryCard", "LockScreenFeatureHintTitle"),
                    SRER.GetValue("Portal", "LibraryCard", "LockScreenFeatureHintMessage"),
                    null);
                return;
            }

            Wpf.Ui.Controls.MessageBoxResult DisableResult = await ShowLockScreenMessageAsync(
                SRER.GetValue("Portal", "LibraryCard", "LockScreenFeatureTitle"),
                SRER.GetValue("Portal", "LibraryCard", "LockScreenFeatureMessage"),
                SRER.GetValue("Portal", "LibraryCard", "LockScreenFeatureDisable"));

            if (DisableResult != Wpf.Ui.Controls.MessageBoxResult.Primary)
            {
                return;
            }

            string ResultFile = Path.Combine(SMMRP.Temp, $"SucroseLockScreen_{Guid.NewGuid():N}.txt");
            string Arguments = $"{SMMRG.StartCommand}{SSDECT.LockScreenFeature}{SMMRG.ValueSeparator}Disable{SMMRG.ValueSeparator}{ResultFile}";

            try
            {
                if (SSSHQ.IsAdministrator())
                {
                    SSSHP.Run(SSSMI.Commandog, Arguments);
                }
                else
                {
                    SSSHP.Runas(SSSMI.Commandog, Arguments);
                }

                if (!await WaitLockScreenResultAsync(ResultFile))
                {
                    await ShowLockScreenMessageAsync(
                        SRER.GetValue("Portal", "LibraryCard", "LockScreenErrorTitle"),
                        SRER.GetValue("Portal", "LibraryCard", "LockScreenTimeoutMessage"),
                        null);
                    return;
                }

                string Outcome = null;

                try
                {
                    Outcome = File.ReadAllText(ResultFile).Split('|').FirstOrDefault();
                }
                catch
                {
                }

                if (Outcome != "OK")
                {
                    string Detail = null;

                    try
                    {
                        string[] Parts = File.ReadAllText(ResultFile).Split('|', 2);
                        Detail = Parts.Length > 1 ? Parts[1] : null;
                    }
                    catch
                    {
                    }

                    await ShowLockScreenMessageAsync(
                        SRER.GetValue("Portal", "LibraryCard", "LockScreenErrorTitle"),
                        string.Format(
                            SRER.GetValue("Portal", "LibraryCard", "LockScreenErrorMessage"),
                            Detail ?? SRER.GetValue("Portal", "LibraryCard", "LockScreenTimeoutMessage")),
                        null);
                    return;
                }

                Wpf.Ui.Controls.MessageBoxResult RebootResult = await ShowLockScreenMessageAsync(
                    SRER.GetValue("Portal", "LibraryCard", "LockScreenRebootTitle"),
                    SRER.GetValue("Portal", "LibraryCard", "LockScreenRebootMessage"),
                    SRER.GetValue("Portal", "LibraryCard", "LockScreenRebootNow"));

                if (RebootResult != Wpf.Ui.Controls.MessageBoxResult.Primary)
                {
                    return;
                }

                try
                {
                    Process.Start(new ProcessStartInfo("shutdown.exe", "/r /t 0") { UseShellExecute = true });
                }
                catch (Exception Exception)
                {
                    await SSWEW.Watch_CatchException(Exception);
                }
            }
            catch (Exception Exception) when (Exception is Win32Exception or OperationCanceledException)
            {
                await SSWEW.Watch_CatchException(Exception);
            }
            finally
            {
                try
                {
                    File.Delete(ResultFile);
                }
                catch
                {
                }
            }
        }

        private static void CleanupLockScreenTemporary(string ResultFile, string VideoTemporary)
        {
            try
            {
                File.Delete(ResultFile);
            }
            catch
            {
            }

            try
            {
                if (!string.IsNullOrEmpty(VideoTemporary) && Directory.Exists(VideoTemporary))
                {
                    Directory.Delete(VideoTemporary, true);
                }
            }
            catch
            {
            }
        }

        private async Task<Wpf.Ui.Controls.MessageBoxResult> ShowLockScreenMessageAsync(string Title, string Message, string Primary)
        {
            MessageBox Dialog = new()
            {
                Title = Title,
                Content = Message,
                CloseButtonText = SRER.GetValue("Portal", "LibraryCard", "LockScreenClose")
            };

            if (!string.IsNullOrEmpty(Primary))
            {
                Dialog.PrimaryButtonText = Primary;
            }

            return await Dialog.ShowDialogAsync();
        }

        private async Task<string> EnsureVideoGifAsync(string Source)
        {
            // Download, probe, options and conversion all run inside the dialog.
            // Closing it cancels whatever stage is in progress.
            LockScreenVideo Options = new()
            {
                SourcePath = Source,
                ThumbnailSource = Path.Combine(Theme, Info.Thumbnail)
            };

            await Options.ShowAsync();

            string Result = Options.ResultGifPath;

            Options.Dispose();

            return Result;
        }

        private void MenuCyclingAdd_Click(object sender, RoutedEventArgs e)
        {
            List<string> Exclusion = SMMC.Exclusion;

            if (Exclusion.Contains(Path.GetFileName(Theme)))
            {
                Exclusion.Remove(Path.GetFileName(Theme));

                SMMI.CyclingSettingManager.SetSetting(SMMCC.Exclusion, Exclusion);
            }
        }

        private async void MenuEdit_Click(object sender, RoutedEventArgs e)
        {
            if (Directory.Exists(Theme))
            {
                SPVCTE ThemeEdit = new()
                {
                    Info = Info,
                    Theme = Theme
                };

                ContentDialogResult Result = await ThemeEdit.ShowAsync();

                if (Result == ContentDialogResult.Primary)
                {
                    Info = SSTHI.ReadJson(Path.Combine(Theme, SMMRC.SucroseInfo));

                    UpdateInfo();
                }

                ThemeEdit.Dispose();
            }
        }

        private async void MenuShare_Click(object sender, RoutedEventArgs e)
        {
            if (Directory.Exists(Theme))
            {
                SPVCTS ThemeShare = new()
                {
                    Info = Info,
                    Theme = Theme
                };

                await ThemeShare.ShowAsync();

                ThemeShare.Dispose();
            }
        }

        private async void MenuReview_Click(object sender, RoutedEventArgs e)
        {
            if (Directory.Exists(Theme))
            {
                SPVCTR ThemeReview = new()
                {
                    Info = Info,
                    Theme = Theme
                };

                await ThemeReview.ShowAsync();

                ThemeReview.Dispose();
            }
        }

        private async void MenuDelete_Click(object sender, RoutedEventArgs e)
        {
            bool Confirm = SMML.DeleteConfirm;

            ContentDialogResult Result = ContentDialogResult.None;

            if (Confirm)
            {
                SPVCTD ThemeDelete = new()
                {
                    Info = Info,
                    Theme = Theme
                };

                Result = await ThemeDelete.ShowAsync();

                ThemeDelete.Dispose();
            }

            if (!Confirm || Result == ContentDialogResult.Primary)
            {
                Dispose();

                MinWidth = 0;
                MinHeight = 0;

                Delete = true;

                Imagine.Source = null;
                Imaginer.Source = null;

                Visibility = Visibility.Hidden;

                await Task.Run(() =>
                {
                    string PropertiesCache = Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.Properties);

                    if (Directory.Exists(PropertiesCache))
                    {
                        foreach (string Record in Directory.GetFiles(PropertiesCache, ""))
                        {
                            if (File.Exists(Record) && Record.Contains(Path.GetFileName(Theme)))
                            {
                                SSSHF.Delete(Record);
                            }
                        }
                    }
                });

                await Task.Run(() =>
                {
                    if (Directory.Exists(Theme))
                    {
                        Directory.Delete(Theme, true);
                    }
                });
            }
        }

        private void MenuCyclingRemove_Click(object sender, RoutedEventArgs e)
        {
            List<string> Exclusion = SMMC.Exclusion;

            if (!Exclusion.Contains(Path.GetFileName(Theme)))
            {
                Exclusion.Add(Path.GetFileName(Theme));

                SMMI.CyclingSettingManager.SetSetting(SMMCC.Exclusion, Exclusion);
            }
        }

        private void ThemeMore_Click(object sender, RoutedEventArgs e)
        {
            ContextMenu.PlacementTarget = this;
            ContextMenu.IsOpen = true;
        }

        private void ContextMenu_Opened(object sender, RoutedEventArgs e)
        {
            MenuUse.Header = SRER.GetValue("Portal", "LibraryCard", "MenuUse");
            MenuDelete.Header = SRER.GetValue("Portal", "LibraryCard", "MenuDelete");
            MenuCustomize.Header = SRER.GetValue("Portal", "LibraryCard", "MenuCustomize");
            MenuLockScreen.Header = SRER.GetValue("Portal", "LibraryCard", "MenuLockScreen");

            string PropertiesPath = Path.Combine(Theme, SMMRC.SucroseProperties);

            if (Info.Type == SSDEWT.Web && File.Exists(PropertiesPath))
            {
                MenuCustomize.IsEnabled = true;
            }
            else if (Info.Type is SSDEWT.Gif or SSDEWT.Video or SSDEWT.YouTube)
            {
                if (Info.Type is SSDEWT.Gif or SSDEWT.Video)
                {
                    if (Info.Type == SSDEWT.Gif)
                    {
                        if (SSDMME.Gif == SSDEET.VlcPlayerLive && (File.Exists(PropertiesPath) || File.Exists(Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.VlcPlayer, SMMRC.SucroseProperties))))
                        {
                            MenuCustomize.IsEnabled = true;
                        }
                        else if (SSDMME.Gif == SSDEET.MpvPlayerLive && (File.Exists(PropertiesPath) || File.Exists(Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.MpvPlayer, SMMRC.SucroseProperties))))
                        {
                            MenuCustomize.IsEnabled = true;
                        }
                        else if (SSDMME.Gif == SSDEET.CefSharpLive && File.Exists(Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.CefSharp, SMMRC.SucroseProperties)))
                        {
                            MenuCustomize.IsEnabled = true;
                        }
                        else if (SSDMME.Gif == SSDEET.WebViewLive && File.Exists(Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.WebView2, SMMRC.SucroseProperties)))
                        {
                            MenuCustomize.IsEnabled = true;
                        }
                        else
                        {
                            MenuCustomize.IsEnabled = false;
                        }
                    }
                    else if (Info.Type == SSDEWT.Video)
                    {
                        if (SSDMME.Video == SSDEET.VlcPlayerLive && (File.Exists(PropertiesPath) || File.Exists(Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.VlcPlayer, SMMRC.SucroseProperties))))
                        {
                            MenuCustomize.IsEnabled = true;
                        }
                        else if (SSDMME.Video == SSDEET.MpvPlayerLive && (File.Exists(PropertiesPath) || File.Exists(Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.MpvPlayer, SMMRC.SucroseProperties))))
                        {
                            MenuCustomize.IsEnabled = true;
                        }
                        else if (SSDMME.Video == SSDEET.CefSharpLive && File.Exists(Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.CefSharp, SMMRC.SucroseProperties)))
                        {
                            MenuCustomize.IsEnabled = true;
                        }
                        else if (SSDMME.Video == SSDEET.WebViewLive && File.Exists(Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.WebView2, SMMRC.SucroseProperties)))
                        {
                            MenuCustomize.IsEnabled = true;
                        }
                        else
                        {
                            MenuCustomize.IsEnabled = false;
                        }
                    }
                    else
                    {
                        MenuCustomize.IsEnabled = false;
                    }
                }
                else if (Info.Type == SSDEWT.YouTube)
                {
                    if (SSDMME.YouTube == SSDEET.CefSharpLive && File.Exists(Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.CefSharp, SMMRC.SucroseProperties)))
                    {
                        MenuCustomize.IsEnabled = true;
                    }
                    else if (SSDMME.YouTube == SSDEET.WebViewLive && File.Exists(Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.WebView2, SMMRC.SucroseProperties)))
                    {
                        MenuCustomize.IsEnabled = true;
                    }
                    else
                    {
                        MenuCustomize.IsEnabled = false;
                    }
                }
                else
                {
                    MenuCustomize.IsEnabled = false;
                }
            }
            else
            {
                MenuCustomize.IsEnabled = false;
            }

            if (SMMC.Active)
            {
                if (SMMC.Exclusion.Contains(Path.GetFileName(Theme)))
                {
                    MenuCyclingAdd.Visibility = Visibility.Visible;
                    MenuCyclingRemove.Visibility = Visibility.Collapsed;
                }
                else
                {
                    MenuCyclingAdd.Visibility = Visibility.Collapsed;
                    MenuCyclingRemove.Visibility = Visibility.Visible;
                }
            }
            else
            {
                MenuCyclingAdd.Visibility = Visibility.Collapsed;
                MenuCyclingRemove.Visibility = Visibility.Collapsed;
            }

            if (Info.Type is SSDEWT.Gif or SSDEWT.Video && Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
            {
                MenuLockScreen.Visibility = Visibility.Visible;
            }
            else
            {
                MenuLockScreen.Visibility = Visibility.Collapsed;
            }

            if ((!SMMB.ClosePerformance && !SMMB.PausePerformance) || !SSSHP.Work(SSSMI.Backgroundog))
            {
                if (SMML.Selected == Path.GetFileName(Theme) && SSSHL.Run())
                {
                    MenuUse.IsEnabled = false;
                    MenuDelete.IsEnabled = false;

                    MenuUse.Header += $" ({SRER.GetValue("Portal", "LibraryCard", "Selected")})";
                    MenuDelete.Header += $" ({SRER.GetValue("Portal", "LibraryCard", "Selected")})";
                }
                else
                {
                    if (Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
                    {
                        MenuUse.IsEnabled = true;
                    }
                    else
                    {
                        MenuUse.IsEnabled = false;

                        MenuUse.Header += $" ({SRER.GetValue("Portal", "LibraryCard", "Incompatible")})";
                    }

                    MenuDelete.IsEnabled = true;
                }
            }
            else
            {
                MenuUse.IsEnabled = false;
                MenuDelete.IsEnabled = false;
                MenuCustomize.IsEnabled = false;
                MenuLockScreen.IsEnabled = false;

                if (SMMB.ClosePerformance)
                {
                    MenuUse.Header += $" ({SRER.GetValue("Portal", "LibraryCard", "Closed")})";
                    MenuDelete.Header += $" ({SRER.GetValue("Portal", "LibraryCard", "Closed")})";
                    MenuCustomize.Header += $" ({SRER.GetValue("Portal", "LibraryCard", "Closed")})";
                }
                else if (SMMB.PausePerformance)
                {
                    MenuUse.Header += $" ({SRER.GetValue("Portal", "LibraryCard", "Paused")})";
                    MenuDelete.Header += $" ({SRER.GetValue("Portal", "LibraryCard", "Paused")})";
                    MenuCustomize.Header += $" ({SRER.GetValue("Portal", "LibraryCard", "Paused")})";

                }
            }
        }

        private void Imaginer_MediaOpened(object sender, RoutedEventArgs e)
        {
            Imaginer.Visibility = Visibility.Visible;
            Imagine.Visibility = Visibility.Hidden;

            if (SMMP.LibraryPreviewHide)
            {
                Preview.Visibility = Visibility.Hidden;
            }

            Dispose();
        }

        private void LibraryCard_MouseLeave(object sender, MouseEventArgs e)
        {
            if (SMMP.LibraryPreview)
            {
                Imaginer.Source = null;
                SXAGAB.SetSourceUri(Imaginer, null);

                Imagine.Visibility = Visibility.Visible;
                Imaginer.Visibility = Visibility.Hidden;

                if (SMMP.LibraryPreviewHide)
                {
                    Preview.Visibility = Visibility.Visible;
                }

                Dispose();
            }
        }

        private void LibraryCard_MouseEnter(object sender, MouseEventArgs e)
        {
            if ((SMML.Selected == Path.GetFileName(Theme) && SSSHL.Run()) || Info.AppVersion.CompareTo(SHV.Entry()) > 0)
            {
                Cursor = Cursors.Arrow;
            }
            else
            {
                Cursor = Cursors.Hand;
            }

            if (SMMP.LibraryPreview)
            {
                string GifPath = Path.Combine(Theme, Info.Preview);

                if (File.Exists(GifPath))
                {
                    SXAGAB.SetSourceUri(Imaginer, new(GifPath));
                    SXAGAB.AddLoadedHandler(Imaginer, Imaginer_MediaOpened);
                }
            }
        }

        private async void LibraryCard_Loaded(object sender, RoutedEventArgs e)
        {
            await Application.Current.Dispatcher.InvokeAsync(async () =>
            {
                UpdateInfo();

                if (Info.AppVersion.CompareTo(SHV.Entry()) > 0)
                {
                    ThemeMore.Visibility = Visibility.Collapsed;
                    IncompatibleVersion.Visibility = Visibility.Visible;
                }

                string ImagePath = Path.Combine(Theme, Info.Thumbnail);

                if (File.Exists(ImagePath))
                {
                    try
                    {
                        Imagine.Source = await Loader.LoadOptimalAsync(ImagePath);
                    }
                    catch (Exception Exception)
                    {
                        await SSWEW.Watch_CatchException(Exception);
                    }
                }

                await Task.Delay(100);

                Card.Visibility = Visibility.Visible;
                Progress.Visibility = Visibility.Collapsed;

                Dispose();
            });
        }

        private void IncompatibleVersion_Click(object sender, RoutedEventArgs e)
        {
            if (!SSSHP.Work(SSSMI.Update))
            {
                SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Update}{SMMRG.ValueSeparator}{SSSMI.Update}");
            }
        }

        private void LibraryCard_MouseLeftButtonUp(object sender, MouseButtonEventArgs e)
        {
            if (Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
            {
                Use();
            }
        }

        public void Dispose()
        {
            Loader.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}