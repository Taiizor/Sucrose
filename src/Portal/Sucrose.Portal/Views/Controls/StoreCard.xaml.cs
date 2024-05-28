using System.IO;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SHA = Skylark.Helper.Adaptation;
using SHG = Skylark.Helper.Generator;
using SHV = Skylark.Helper.Versionly;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SPEIL = Sucrose.Portal.Extension.ImageLoader;
using SPMI = Sucrose.Portal.Manage.Internal;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSDEST = Sucrose.Shared.Dependency.Enum.StoreType;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSSHC = Sucrose.Shared.Space.Helper.Copy;
using SSSHGHD = Sucrose.Shared.Store.Helper.GitHub.Download;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSHS = Sucrose.Shared.Store.Helper.Store;
using SSSHSD = Sucrose.Shared.Store.Helper.Soferity.Download;
using SSSID = Sucrose.Shared.Store.Interface.Data;
using SSSIW = Sucrose.Shared.Store.Interface.Wallpaper;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SSSPMI = Sucrose.Shared.Space.Manage.Internal;
using SSSTMI = Sucrose.Shared.Store.Manage.Internal;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SXAGAB = Sucrose.XamlAnimatedGif.AnimationBehavior;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// StoreCard.xaml etkileşim mantığı
    /// </summary>
    public partial class StoreCard : UserControl, IDisposable
    {
        private readonly KeyValuePair<string, SSSIW> Wallpaper = new();
        private readonly string Theme = string.Empty;
        private readonly SPEIL Loader = new();
        private string Keys = string.Empty;
        private readonly string Agent;
        private readonly string Guid;
        private readonly string Key;
        private SSTHI Info;
        private bool State;

        internal StoreCard(string Theme, KeyValuePair<string, SSSIW> Wallpaper, string Agent, string Key)
        {
            this.Key = Key;
            this.Agent = Agent;
            this.Theme = Theme;
            this.Wallpaper = Wallpaper;
            this.Guid = Path.Combine(Wallpaper.Value.Source, Wallpaper.Key);

            InitializeComponent();
        }

        private async void Start()
        {
            if (Info != null && Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
            {
                if (DownloadSymbol.Symbol == SymbolRegular.CloudArrowDown24)
                {
                    if (SSSHN.GetHostEntry())
                    {
                        State = true;

                        DownloadSymbol.Symbol = SymbolRegular.Empty;

                        DownloadRing.Visibility = Visibility.Visible;
                        DownloadSymbol.Visibility = Visibility.Collapsed;

                        await Task.Run(DownloadTheme);
                    }
                    else
                    {
                        DownloadSymbol.Foreground = SRER.GetResource<Brush>("PaletteRedBrush");
                        DownloadSymbol.Symbol = SymbolRegular.CloudDismiss24;

                        await Task.Delay(3000);

                        DownloadSymbol.Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush");
                        DownloadSymbol.Symbol = SymbolRegular.CloudArrowDown24;
                    }
                }
            }
            else
            {
                DownloadSymbol.Visibility = Visibility.Hidden;
                IncompatibleVersion.Visibility = Visibility.Visible;
            }
        }

        private void MenuReport_Click(object sender, RoutedEventArgs e)
        {
            string Title = Wallpaper.Key.Replace(" ", "%20");
            string Location = $"{Wallpaper.Value.Source.Replace(" ", "%20").Split('/').LastOrDefault()}/{Title}";

            SSSHP.Run(SSSPMI.Commandog, $"{SMR.StartCommand}{SSDECT.Report}{SMR.ValueSeparator}{SMR.StoreReportWebsite}&title={Title}&wallpaper-location={Location}");
        }

        private void MenuInstall_Click(object sender, RoutedEventArgs e)
        {
            Start();
        }

        private async Task<bool> DownloadCache()
        {
            if (SPMI.StoreDownloader.ContainsKey(Theme))
            {
                while (!SPMI.StoreDownloading.ContainsKey(Theme) || !SPMI.StoreDownloading[Theme])
                {
                    await Task.Delay(100);
                }

                Info = SSTHI.ReadJson(Path.Combine(Theme, SMR.SucroseInfo));

                return true;
            }
            else
            {
                SPMI.StoreDownloader[Theme] = false;

                SPMI.StoreDownloader[Theme] = SSDMM.StoreType switch
                {
                    SSDEST.GitHub => SSSHGHD.Cache(Wallpaper, Theme, Agent, Key),
                    _ => SSSHSD.Cache(Wallpaper, Theme, Agent),
                };

                if (SPMI.StoreDownloader[Theme])
                {
                    while (!SPMI.StoreDownloading.ContainsKey(Theme) || !SPMI.StoreDownloading[Theme])
                    {
                        await Task.Delay(100);
                    }

                    Info = SSTHI.ReadJson(Path.Combine(Theme, SMR.SucroseInfo));

                    return true;
                }
                else
                {
                    return false;
                }
            }
        }

        private async void DownloadTheme()
        {
            do
            {
                Keys = SHG.GenerateString(SMMM.Chars, 25, SMR.Randomise);
            } while (File.Exists(Path.Combine(SMMM.LibraryLocation, Keys)));

            SSSTMI.StoreService.InfoChanged += (s, e) => StoreService_InfoChanged(Keys);

            string LibraryPath = Path.Combine(SMMM.LibraryLocation, Keys);
            string TemporaryPath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Store, SMR.Temporary, Keys);

            switch (SSDMM.StoreType)
            {
                case SSDEST.GitHub:
                    await SSSHGHD.Theme(Path.Combine(Wallpaper.Value.Source, Wallpaper.Key), TemporaryPath, Agent, Guid, Keys, Key);
                    break;
                default:
                    await SSSHSD.Theme(Path.Combine(Wallpaper.Value.Source, Wallpaper.Key), TemporaryPath, Agent, Guid, Keys);
                    break;
            }

            await Task.Delay(100);

            if (Directory.Exists(TemporaryPath))
            {
                SSSHC.Folder(TemporaryPath, LibraryPath);

                if ((!SMMM.ClosePerformance && !SMMM.PausePerformance) || !SSSHP.Work(SSSMI.Backgroundog))
                {
                    if (SMMM.StoreStart)
                    {
                        SMMI.LibrarySettingManager.SetSetting(SMC.LibrarySelected, Path.GetFileName(Keys));

                        if (SSSHL.Run())
                        {
                            SSLHK.Stop();
                        }

                        SSLHR.Start();
                    }
                }
            }
        }

        private void Download_Click(object sender, RoutedEventArgs e)
        {
            Start();
        }

        private async void StoreService_InfoChanged(string Keys)
        {
            if (this.Keys == Keys && SSSTMI.StoreService.Info.ContainsKey(Keys))
            {
                await Application.Current.Dispatcher.InvokeAsync(async () =>
                {
                    DownloadRing.Progress = SSSTMI.StoreService.Info[Keys].ProgressPercentage;

                    ToolTip RingTip = new()
                    {
                        Content = SSSTMI.StoreService.Info[Keys].Percentage
                    };

                    Download.ToolTip = RingTip;

                    if (SSSTMI.StoreService.Info[Keys].ProgressPercentage >= 100)
                    {
                        if (State)
                        {
                            State = false;

                            await Task.Delay(1500);

                            Download.ToolTip = null;
                            DownloadRing.Visibility = Visibility.Collapsed;
                            DownloadSymbol.Visibility = Visibility.Visible;

                            DownloadSymbol.Foreground = SRER.GetResource<Brush>("SystemFillColorSuccessBrush");
                            DownloadSymbol.Symbol = SymbolRegular.CloudCheckmark24;

                            await Task.Delay(3000);

                            DownloadSymbol.Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush");
                            DownloadSymbol.Symbol = SymbolRegular.CloudArrowDown24;
                        }
                    }
                });
            }
        }

        private void ContextMenu_Opened(object sender, RoutedEventArgs e)
        {
            if (Info != null)
            {
                MenuReport.IsEnabled = true;
            }
            else
            {
                MenuReport.IsEnabled = false;
            }

            if (DownloadSymbol.Symbol == SymbolRegular.CloudArrowDown24 && Info != null && Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
            {
                MenuInstall.IsEnabled = true;
            }
            else
            {
                MenuInstall.IsEnabled = false;
            }
        }

        private void StoreCard_MouseLeave(object sender, MouseEventArgs e)
        {
            if (Info != null && SMMM.StorePreview)
            {
                Imaginer.Source = null;
                SXAGAB.SetSourceUri(Imaginer, null);

                Imagine.Visibility = Visibility.Visible;
                Imaginer.Visibility = Visibility.Hidden;

                if (SMMM.StorePreviewHide)
                {
                    Preview.Visibility = Visibility.Visible;
                }

                Dispose();
            }
        }

        private void StoreCard_MouseEnter(object sender, MouseEventArgs e)
        {
            if (Info != null && SMMM.StorePreview)
            {
                string GifPath = $"{SSSHS.Source(SSDMM.StoreType)}/{Wallpaper.Value.Source}/{Wallpaper.Key}/{Wallpaper.Value.Live}";

                SXAGAB.SetSourceUri(Imaginer, new(GifPath));
                SXAGAB.AddLoadedHandler(Imaginer, Imaginer_MediaOpened);
            }
        }

        private void Imaginer_MediaOpened(object sender, RoutedEventArgs e)
        {
            Imaginer.Visibility = Visibility.Visible;
            Imagine.Visibility = Visibility.Hidden;

            if (SMMM.StorePreviewHide)
            {
                Preview.Visibility = Visibility.Hidden;
            }

            Dispose();
        }

        private async void StoreCard_Loaded(object sender, RoutedEventArgs e)
        {
            try
            {
                bool Result = await Task.Run(DownloadCache);

                if (Result)
                {
                    ToolTip TitleTip = new()
                    {
                        Content = Info.Title
                    };

                    ToolTip DescriptionTip = new()
                    {
                        Content = Info.Description
                    };

                    ThemeTitle.ToolTip = TitleTip;
                    ThemeDescription.ToolTip = DescriptionTip;

                    ThemeTitle.Text = Info.Title.Length > SMMM.TitleLength ? $"{SHA.Cut(Info.Title, SMMM.TitleLength)}..." : Info.Title;
                    ThemeDescription.Text = Info.Description.Length > SMMM.DescriptionLength ? $"{SHA.Cut(Info.Description, SMMM.DescriptionLength)}..." : Info.Description;

                    string ImagePath = Path.Combine(Theme, Info.Thumbnail);

                    if (File.Exists(ImagePath))
                    {
                        Imagine.Source = await Loader.LoadOptimalAsync(ImagePath);
                    }

                    if (Info.AppVersion.CompareTo(SHV.Entry()) > 0)
                    {
                        DownloadSymbol.Visibility = Visibility.Hidden;
                        IncompatibleVersion.Visibility = Visibility.Visible;
                    }

                    foreach (KeyValuePair<string, SSSID> Pair in SSSTMI.StoreService.Info.ToList())
                    {
                        if (Pair.Value.Guid == Guid)
                        {
                            Keys = Pair.Key;

                            if (SSSTMI.StoreService.Info[Keys].ProgressPercentage < 100)
                            {
                                State = true;

                                StoreService_InfoChanged(Keys);

                                DownloadSymbol.Symbol = SymbolRegular.Empty;

                                DownloadRing.Visibility = Visibility.Visible;
                                DownloadSymbol.Visibility = Visibility.Collapsed;

                                SSSTMI.StoreService.InfoChanged += (s, e) => StoreService_InfoChanged(Keys);

                                break;
                            }
                        }
                    }

                    await Task.Delay(100);

                    Card.Visibility = Visibility.Visible;
                    Progress.Visibility = Visibility.Collapsed;
                }
                else
                {
                    Warn.Visibility = Visibility.Visible;
                    Progress.Visibility = Visibility.Collapsed;
                }

                Dispose();
            }
            catch
            {
                Warn.Visibility = Visibility.Visible;
                Progress.Visibility = Visibility.Collapsed;

                Dispose();
            }
        }

        private void StoreCard_MouseLeftButtonUp(object sender, MouseButtonEventArgs e)
        {
            Start();
        }

        public void Dispose()
        {
            Loader.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}