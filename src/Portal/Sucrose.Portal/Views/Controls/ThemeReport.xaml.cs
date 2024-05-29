using Newtonsoft.Json;
using System.IO;
using System.Net.Http;
using System.Text;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using Wpf.Ui.Controls;
using Button = Wpf.Ui.Controls.Button;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SPEIL = Sucrose.Portal.Extension.ImageLoader;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPMRD = Sucrose.Portal.Models.ReportData;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSDERT = Sucrose.Shared.Dependency.Enum.ReportType;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSHU = Sucrose.Shared.Space.Helper.User;
using SSSIW = Sucrose.Shared.Store.Interface.Wallpaper;
using SSSPMI = Sucrose.Shared.Space.Manage.Internal;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using TextBlock = Wpf.Ui.Controls.TextBlock;
using TextBox = Wpf.Ui.Controls.TextBox;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeReport.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeReport : ContentDialog, IDisposable
    {
        internal KeyValuePair<string, SSSIW> Wallpaper = new();
        private Button ReporterReport = new();
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

            ComboBox ReportMode = new()
            {
                MaxDropDownHeight = 200
            };

            ScrollViewer.SetVerticalScrollBarVisibility(ReportMode, ScrollBarVisibility.Auto);

            foreach (SSDERT Type in Enum.GetValues(typeof(SSDERT)))
            {
                ReportMode.Items.Add(new ComboBoxItem()
                {
                    Content = SRER.GetValue("Portal", "Enum", "ReportType", $"{Type}"),
                    Tag = $"{Type}"
                });
            }

            ReportMode.SelectedIndex = (int)SSDERT.Other;

            Reporter.HeaderFrame = ReportMode;

            StackPanel ReporterContent = new();

            TextBlock ReporterDescriptionText = new()
            {
                Text = SRER.GetValue("Portal", "ThemeReport", "Reporter", "Description", "Text"),
                Foreground = SRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
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

            TextBlock ReporterState = new()
            {
                Text = SRER.GetValue("Portal", "ThemeReport", "Reporter", "Reporting", "Preparing"),
                Foreground = SRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                HorizontalAlignment = HorizontalAlignment.Center,
                VerticalAlignment = VerticalAlignment.Center,
                TextAlignment = TextAlignment.Center,
                Margin = new Thickness(0, 0, 0, 10),
                Visibility = Visibility.Collapsed,
                TextWrapping = TextWrapping.Wrap,
                FontSize = 14
            };

            Grid ReporterCustomContent = new();

            ReporterReport = new()
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
                if (string.IsNullOrEmpty(ReportDescription.Text) || string.IsNullOrWhiteSpace(ReportDescription.Text))
                {
                    ReportDescription.Focus();
                }
                else
                {
                    string Description = ReportDescription.Text;
                    string Title = Wallpaper.Key.Replace(" ", "%20");
                    string Location = $"{Wallpaper.Value.Source.Replace(" ", "%20").Split('/').LastOrDefault()}/{Title}";

                    SSSHP.Run(SSSPMI.Commandog, $"{SMR.StartCommand}{SSDECT.Report}{SMR.ValueSeparator}{SMR.StoreReportWebsite}&title={Title}&wallpaper-location={Location}&report-description={Description}");
                }
            };

            ReporterCustomContent.Children.Add(ReporterReport);
            ReporterCustomContent.Children.Add(ReporterGitHub);

            ReporterContent.Children.Add(ReporterDescriptionText);
            ReporterContent.Children.Add(ReportDescription);
            ReporterContent.Children.Add(ReporterState);
            ReporterContent.Children.Add(ReporterCustomContent);

            Reporter.FooterCard = ReporterContent;

            ReporterReport.Click += async (s, e) =>
            {
                if (string.IsNullOrEmpty(ReportDescription.Text) || string.IsNullOrWhiteSpace(ReportDescription.Text))
                {
                    ReportDescription.Focus();
                }
                else
                {
                    ReporterReport.IsEnabled = false;
                    ReporterGitHub.IsEnabled = false;
                    ReporterState.Visibility = Visibility.Visible;

                    if (await SSSHN.GetHostEntryAsync())
                    {
                        using HttpClient Client = new();

                        HttpResponseMessage Response = new();

                        Client.DefaultRequestHeaders.Add("User-Agent", SMMM.UserAgent);

                        ReporterState.Text = SRER.GetValue("Portal", "ThemeReport", "Reporter", "Checking");

                        await Task.Delay(1000);

                        try
                        {
                            Response = await Client.GetAsync($"{SMR.SoferityWebsite}/{SMR.SoferityReport}/{SMR.Check}/{SSSHU.GetGuid()}");
                        }
                        catch
                        {
                            ReporterState.Text = SRER.GetValue("Portal", "ThemeReport", "Reporter", "Checking", "Error");
                        }

                        if (Response.IsSuccessStatusCode)
                        {
                            ReporterState.Text = SRER.GetValue("Portal", "ThemeReport", "Reporter", "Sending");

                            await Task.Delay(1000);

                            Response = new();

                            try
                            {
                                SPMRD ReportData = new(Wallpaper.Key, $"{(ReportMode.SelectedItem as ComboBoxItem).Tag}", $"{Wallpaper.Value.Source.Split('/').LastOrDefault()}/{Wallpaper.Key}", ReportDescription.Text);

                                StringContent Content = new(JsonConvert.SerializeObject(ReportData, Formatting.Indented), Encoding.UTF8, "application/json");

                                Response = await Client.PostAsync($"{SMR.SoferityWebsite}/{SMR.SoferityReport}/{SMR.Theme}/{SSSHU.GetGuid()}", Content);
                            }
                            catch
                            {
                                ReporterState.Text = SRER.GetValue("Portal", "ThemeReport", "Reporter", "Sending", "Error");
                            }

                            if (Response.IsSuccessStatusCode)
                            {
                                ReporterState.Text = SRER.GetValue("Portal", "ThemeReport", "Reporter", "Sending", "Succeded");
                            }
                            else
                            {
                                ReporterState.Text = string.Format(SRER.GetValue("Portal", "ThemeReport", "Reporter", "Sending", "Errored"), Response.StatusCode);
                            }
                        }
                        else
                        {
                            ReporterState.Text = SRER.GetValue("Portal", "ThemeReport", "Reporter", "Checking", "Status");
                        }
                    }
                    else
                    {
                        ReporterState.Text = SRER.GetValue("Portal", "ThemeReport", "Reporter", "Network");
                    }

                    await Task.Delay(3000);

                    ReporterReport.IsEnabled = true;
                    ReporterGitHub.IsEnabled = true;
                }
            };
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
            if (!ReporterReport.IsEnabled)
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