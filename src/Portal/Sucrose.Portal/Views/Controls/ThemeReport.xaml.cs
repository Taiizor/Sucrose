using Sucrose.Shared.Store.Interface;
using System.IO;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media.Imaging;
using Wpf.Ui.Controls;
using SaveFileDialog = Microsoft.Win32.SaveFileDialog;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SPEIL = Sucrose.Portal.Extension.ImageLoader;
using SPMI = Sucrose.Portal.Manage.Internal;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDECYT = Sucrose.Shared.Dependency.Enum.CompatibilityType;
using SSDEST = Sucrose.Shared.Dependency.Enum.StoreType;
using SSDERT = Sucrose.Shared.Dependency.Enum.ReportType;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using Button = Wpf.Ui.Controls.Button;
using SSSEPS = Sucrose.Shared.Space.Extension.ProgressStream;
using SSSHC = Sucrose.Shared.Space.Helper.Clean;
using SSSHGHD = Sucrose.Shared.Store.Helper.GitHub.Download;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHS = Sucrose.Shared.Store.Helper.Store;
using SSSHSD = Sucrose.Shared.Store.Helper.Soferity.Download;
using SSSHU = Sucrose.Shared.Space.Helper.User;
using SSSIR = Sucrose.Shared.Store.Interface.Root;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSZEZ = Sucrose.Shared.Zip.Extension.Zip;
using SSSIW = Sucrose.Shared.Store.Interface.Wallpaper;
using SSDECST = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSSPMI = Sucrose.Shared.Space.Manage.Internal;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using TextBox = Wpf.Ui.Controls.TextBox;
using TextBlock = Wpf.Ui.Controls.TextBlock;
using System.Windows.Media;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeReport.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeReport : ContentDialog, IDisposable
    {
        private ComboBox ReportMode => (ComboBox)Reporter.HeaderFrame;
        internal KeyValuePair<string, SSSIW> Wallpaper = new();
        private readonly SPEIL Loader = new();
        internal string Theme = string.Empty;
        internal SSTHI Info = new();

        public ThemeReport() : base(SPMI.ContentDialogService.GetContentPresenter())
        {
            InitializeComponent();
        }

        private void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            string ImagePath = Path.Combine(Theme, Info.Thumbnail);

            if (File.Exists(ImagePath))
            {
                ThemeThumbnail.Source = Loader.LoadOptimal(ImagePath);
            }

            ThemeTitle.Text = Info.Title;
            ThemeDescription.Text = Info.Description;

            Reporter.LeftIcon.Symbol = SymbolRegular.Info24;
            Reporter.Title.Text = SRER.GetValue("Portal", "ThemeReport", "Reporter");
            Reporter.Description.Text = SRER.GetValue("Portal", "ThemeReport", "Reporter", "Description");

            foreach (SSDERT Type in Enum.GetValues(typeof(SSDERT)))
            {
                ReportMode.Items.Add(new ComboBoxItem()
                {
                    Content = SRER.GetValue("Portal", "Enum", "ReportType", $"{Type}"),
                    Tag = $"{Type}"
                });
            }

            ReportMode.SelectedIndex = (int)SSDERT.Other;

            StackPanel ReporterContent = new();

            TextBlock ReporterDescriptionText = new()
            {
                Text = SRER.GetValue("Portal", "ThemeReport", "Reporter", "Description", "Text"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 0, 10),
                FontWeight = FontWeights.SemiBold
            };

            TextBox ReportDescription = new()
            {
                PlaceholderText = SRER.GetValue("Portal", "ThemeReport", "Reporter", "Description", "Placeholder"),
                TextWrapping = TextWrapping.WrapWithOverflow,
                Margin = new Thickness(0, 0, 0, 10),
                AcceptsReturn = false,
                AcceptsTab = false,
                MaxLength = 2500,
                MaxLines = 10,
                MinLines = 5,
            };

            Grid ReporterCustomContent = new();

            Button ReporterReport = new()
            {
                Content = SRER.GetValue("Portal", "ThemeReport", "Reporter", "Report"),
                HorizontalAlignment = HorizontalAlignment.Left,
                Appearance = ControlAppearance.Primary,
                Cursor = Cursors.Hand
            };

            Button ReporterGitHub = new()
            {
                Content = SRER.GetValue("Portal", "ThemeReport", "Reporter", "GitHub"),
                HorizontalAlignment = HorizontalAlignment.Right,
                Appearance = ControlAppearance.Secondary,
                Cursor = Cursors.Hand
            };

            ReporterGitHub.Click += (s, e) =>
            {
                string Title = Wallpaper.Key.Replace(" ", "%20");
                string Location = $"{Wallpaper.Value.Source.Replace(" ", "%20").Split('/').LastOrDefault()}/{Title}";

                SSSHP.Run(SSSPMI.Commandog, $"{SMR.StartCommand}{SSDECST.Report}{SMR.ValueSeparator}{SMR.StoreReportWebsite}&title={Title}&wallpaper-location={Location}");
            };

            ReporterCustomContent.Children.Add(ReporterReport);
            ReporterCustomContent.Children.Add(ReporterGitHub);

            ReporterContent.Children.Add(ReporterDescriptionText);
            ReporterContent.Children.Add(ReportDescription);

            ReporterContent.Children.Add(ReporterCustomContent);

            Reporter.FooterCard = ReporterContent;
        }

        private void ContentDialog_PreviewKeyDown(object sender, KeyEventArgs e)
        {
            if (e.Key is Key.Enter or Key.Escape)
            {
                e.Handled = true;
            }
        }

        protected override void OnButtonClick(ContentDialogButton Button)
        {
            if (!true)
            {
                return;
            }

            base.OnButtonClick(Button);
        }

        public void Dispose()
        {
            Loader.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}