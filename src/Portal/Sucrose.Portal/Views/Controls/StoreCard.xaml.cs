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
using SMR = Sucrose.Memory.Readonly;
using SPEIL = Sucrose.Portal.Extension.ImageLoader;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPMM = Sucrose.Portal.Manage.Manager;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SSSHD = Sucrose.Shared.Store.Helper.Download;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSIW = Sucrose.Shared.Store.Interface.Wallpaper;
using SSSPMI = Sucrose.Shared.Space.Manage.Internal;
using SSSTMI = Sucrose.Shared.Store.Manage.Internal;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// StoreCard.xaml etkileşim mantığı
    /// </summary>
    public partial class StoreCard : UserControl, IDisposable
    {
        private readonly KeyValuePair<string, SSSIW> Wallpaper = new();
        private readonly SPEIL Loader = new();
        private readonly string Theme = null;
        private string Keys = string.Empty;
        private readonly string Agent;
        private readonly string Key;
        private SSTHI Info;
        private bool State;

        internal StoreCard(string Theme, KeyValuePair<string, SSSIW> Wallpaper, string Agent, string Key)
        {
            this.Key = Key;
            this.Agent = Agent;
            this.Theme = Theme;
            this.Wallpaper = Wallpaper;

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
                        DownloadSymbol.Foreground = SSRER.GetResource<Brush>("PaletteRedBrush");
                        DownloadSymbol.Symbol = SymbolRegular.CloudDismiss24;

                        await Task.Delay(3000);

                        DownloadSymbol.Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush");
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

            SSSHP.Run(SSSPMI.Commandog, $"{SMR.StartCommand}{SSDECT.Report}{SMR.ValueSeparator}{SMR.WallpaperReportWebsite}&title={Title}&wallpaper-location={Location}");
        }

        private void MenuInstall_Click(object sender, RoutedEventArgs e)
        {
            Start();
        }

        private async void DownloadCache()
        {
            if (SPMI.StoreDownloader.ContainsKey(Theme))
            {
                while (!SPMI.StoreDownloading.ContainsKey(Theme) || !SPMI.StoreDownloading[Theme])
                {
                    await Task.Delay(100);
                }

                Info = SSTHI.ReadJson(Path.Combine(Theme, SMR.SucroseInfo));
            }
            else
            {
                SPMI.StoreDownloader[Theme] = true;

                SSSHD.Cache(Wallpaper, Theme, Agent, Key);

                while (!SPMI.StoreDownloading.ContainsKey(Theme) || !SPMI.StoreDownloading[Theme])
                {
                    await Task.Delay(100);
                }

                Info = SSTHI.ReadJson(Path.Combine(Theme, SMR.SucroseInfo));
            }
        }

        private async void DownloadTheme()
        {
            do
            {
                Keys = SHG.GenerateString(SPMM.Chars, 25, SMR.Randomise);
            } while (File.Exists(Path.Combine(SPMM.LibraryLocation, Keys)));

            SSSTMI.StoreService.InfoChanged += (s, e) => StoreService_InfoChanged(Keys);

            string LibraryPath = Path.Combine(SPMM.LibraryLocation, Keys);
            string TemporaryPath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Store, SMR.Temporary, Keys);

            await SSSHD.Theme(Path.Combine(Wallpaper.Value.Source, Wallpaper.Key), TemporaryPath, Agent, Keys, Key);

            await Task.Delay(100);

            if (Directory.Exists(TemporaryPath))
            {
                CopyDirectory(TemporaryPath, LibraryPath);

                if (SPMM.Start)
                {
                    SMMI.LibrarySettingManager.SetSetting(SMC.LibrarySelected, Path.GetFileName(Keys));

                    if (SSSHL.Run())
                    {
                        SSSHL.Kill();
                    }

                    SSLHR.Start();
                }
            }
        }

        private void Download_Click(object sender, RoutedEventArgs e)
        {
            Start();
        }

        private async void StoreService_InfoChanged(string Keys)
        {
            if (this.Keys == Keys)
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

                            DownloadSymbol.Foreground = SSRER.GetResource<Brush>("SystemFillColorSuccessBrush");
                            DownloadSymbol.Symbol = SymbolRegular.CloudCheckmark24;

                            await Task.Delay(3000);

                            DownloadSymbol.Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush");
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

        private async void StoreCard_Loaded(object sender, RoutedEventArgs e)
        {
            await Task.Run(DownloadCache);

            try
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

                ThemeTitle.Text = Info.Title.Length > SPMM.TitleLength ? $"{SHA.Cut(Info.Title, SPMM.TitleLength)}..." : Info.Title;
                ThemeDescription.Text = Info.Description.Length > SPMM.DescriptionLength ? $"{SHA.Cut(Info.Description, SPMM.DescriptionLength)}..." : Info.Description;

                string ImagePath = Path.Combine(Theme, Info.Thumbnail);

                if (File.Exists(ImagePath))
                {
                    Imagine.ImageSource = await Loader.LoadOptimalAsync(ImagePath);
                }

                if (Info.AppVersion.CompareTo(SHV.Entry()) > 0)
                {
                    DownloadSymbol.Visibility = Visibility.Hidden;
                    IncompatibleVersion.Visibility = Visibility.Visible;
                }

                await Task.Delay(100);

                Card.Visibility = Visibility.Visible;
                Progress.Visibility = Visibility.Collapsed;

                Dispose();
            }
            catch
            {
                Dispose();
            }
        }

        private void StoreCard_MouseLeftButtonUp(object sender, MouseButtonEventArgs e)
        {
            Start();
        }

        private static void CopyDirectory(string Source, string Destination, bool Delete = true)
        {
            if (!Directory.Exists(Destination))
            {
                Directory.CreateDirectory(Destination);
            }

            foreach (string Record in Directory.GetFiles(Source))
            {
                string DestinationFile = Path.Combine(Destination, Path.GetFileName(Record));
                File.Copy(Record, DestinationFile, true);
            }

            foreach (string SubDirectory in Directory.GetDirectories(Source))
            {
                string DestinationSubDirectory = Path.Combine(Destination, Path.GetFileName(SubDirectory));
                CopyDirectory(SubDirectory, DestinationSubDirectory);
            }

            if (Delete)
            {
                Directory.Delete(Source, true);
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