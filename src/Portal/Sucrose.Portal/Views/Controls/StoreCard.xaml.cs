using System.IO;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SHA = Skylark.Helper.Adaptation;
using SHG = Skylark.Helper.Generator;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SPEIL = Sucrose.Portal.Extension.ImageLoader;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPMM = Sucrose.Portal.Manage.Manager;
using SPVCTR = Sucrose.Portal.Views.Controls.ThemeReview;
using SPVCTS = Sucrose.Portal.Views.Controls.ThemeShare;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SSSHD = Sucrose.Shared.Store.Helper.Download;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSIW = Sucrose.Shared.Store.Interface.Wallpaper;
using SSSMI = Sucrose.Shared.Store.Manage.Internal;
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

                    DownloadSymbol.Foreground = SSRER.GetResource<Brush>("PaletteBlueBrush");
                    DownloadSymbol.Symbol = SymbolRegular.CloudArrowDown24;
                }
            }
        }

        private void MenuUse_Click(object sender, RoutedEventArgs e)
        {
            //
        }

        private void MenuFind_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(Theme);
        }

        private async void MenuShare_Click(object sender, RoutedEventArgs e)
        {
            SPVCTS ThemeShare = new()
            {
                Info = Info,
                Theme = Theme
            };
            await ThemeShare.ShowAsync();
            ThemeShare.Dispose();
        }

        private async void MenuReview_Click(object sender, RoutedEventArgs e)
        {
            SPVCTR ThemeReview = new()
            {
                Info = Info,
                Theme = Theme
            };
            await ThemeReview.ShowAsync();
            ThemeReview.Dispose();
        }

        private async void MenuDelete_Click(object sender, RoutedEventArgs e)
        {
            Dispose();
            MinWidth = 0;
            MinHeight = 0;

            Imagine.ImageSource = null;

            Visibility = Visibility.Hidden;

            await Task.Run(() => Directory.Delete(Theme, true));
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

            SSSMI.StoreService.InfoChanged += (s, e) => StoreService_InfoChanged(Keys);

            await SSSHD.Theme(Path.Combine(Wallpaper.Value.Source, Wallpaper.Key), Path.Combine(SPMM.LibraryLocation, Keys), Agent, Keys, Key);
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
                    DownloadRing.Progress = SSSMI.StoreService.Info[Keys].ProgressPercentage;

                    if (SSSMI.StoreService.Info[Keys].ProgressPercentage >= 100)
                    {
                        if (State)
                        {
                            State = false;

                            await Task.Delay(1500);

                            DownloadRing.Visibility = Visibility.Collapsed;
                            DownloadSymbol.Visibility = Visibility.Visible;

                            DownloadSymbol.Foreground = SSRER.GetResource<Brush>("PaletteGreenBrush");
                            DownloadSymbol.Symbol = SymbolRegular.CloudCheckmark24;

                            await Task.Delay(3000);

                            DownloadSymbol.Foreground = SSRER.GetResource<Brush>("PaletteBlueBrush");
                            DownloadSymbol.Symbol = SymbolRegular.CloudArrowDown24;
                        }
                    }
                });
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
                    Imagine.ImageSource = Loader.Load(ImagePath);
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

        private void ContextMenu_Opened(object sender, RoutedEventArgs e)
        {
            if (SPMM.LibrarySelected == Path.GetFileName(Theme) && SSSHL.Run())
            {
                MenuUse.IsEnabled = false;
                MenuDelete.IsEnabled = false;
            }
            else
            {
                MenuUse.IsEnabled = true;
                MenuDelete.IsEnabled = true;
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