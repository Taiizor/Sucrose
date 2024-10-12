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
using SMMG = Sucrose.Manager.Manage.General;
using SMMO = Sucrose.Manager.Manage.Objectionable;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMR = Sucrose.Memory.Readonly;
using SPEIL = Sucrose.Portal.Extension.ImageLoader;
using SPMI = Sucrose.Portal.Manage.Internal;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDECT = Sucrose.Shared.Dependency.Enum.CompatibilityType;
using SSDESST = Sucrose.Shared.Dependency.Enum.StoreServerType;
using SSDMMP = Sucrose.Shared.Dependency.Manage.Manager.Portal;
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
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRU = Sucrose.Memory.Manage.Readonly.Url;
using SSZHA = Sucrose.Shared.Zip.Helper.Archive;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeShare.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeShare : ContentDialog, IDisposable
    {
        private readonly SPEIL Loader = new();
        internal string Theme = string.Empty;
        internal SSTHI Info = new();

        public ThemeShare() : base(SPMI.ContentDialogService.GetDialogHost())
        {
            InitializeComponent();
        }

        private BitmapImage LoadImage(string Image)
        {
            BitmapImage Back = new();

            Back.BeginInit();

            Back.UriSource = new Uri($"pack://application:,,,/Assets/Icon/{Image}.png", UriKind.RelativeOrAbsolute);

            Back.EndInit();

            return Back;
        }

        private async void Export_Click(object sender, RoutedEventArgs e)
        {
            Export.IsEnabled = false;

            await Task.Run(() =>
            {
                SaveFileDialog SaveDialog = new()
                {
                    FileName = SSSHC.FileName(Info.Title),

                    Filter = SRER.GetValue("Portal", "ThemeShare", "SaveDialogFilter"),
                    FilterIndex = 1,

                    Title = SRER.GetValue("Portal", "ThemeShare", "SaveDialogTitle"),

                    InitialDirectory = SMMRP.Desktop
                };

                if (SaveDialog.ShowDialog() == true)
                {
                    string Destination = SaveDialog.FileName;

                    SSZEZ.Compress(Theme, Destination);
                }
            });

            Export.IsEnabled = true;
        }

        private async void Publish_Click(object sender, RoutedEventArgs e)
        {
            Publish.IsEnabled = false;

            if (PublishGrid.Visibility == Visibility.Collapsed)
            {
                if (await SSSHN.GetHostEntryAsync())
                {
                    Image.Source = LoadImage("Loading");

                    await Task.Delay(500);

                    string StoreFile = Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMR.CacheFolder, SMR.Store, SMR.StoreFile);

                    bool Result = SSDMMP.StoreServerType switch
                    {
                        SSDESST.GitHub => SSSHGHD.Store(StoreFile, SMMG.UserAgent, SMMO.Key),
                        _ => SSSHSD.Store(StoreFile, SMMG.UserAgent),
                    };

                    if (Result)
                    {
                        Image.Source = LoadImage("Basket");

                        SSSIR Root = SSSHS.DeserializeRoot(StoreFile);

                        List<ComboBoxItem> Categories = new();

                        foreach (string Key in Root.Categories.Keys)
                        {
                            Categories.Add(new ComboBoxItem()
                            {
                                Content = SRER.GetValue("Portal", "Category", Key.Replace(" ", "")),
                                Tag = Key
                            });
                        }

                        Category.ItemsSource = Categories.OrderBy(Item => Item.Content);

                        Category.SelectedIndex = 0;

                        PublishGrid.Visibility = Visibility.Visible;
                    }
                    else
                    {
                        Image.Source = LoadImage("Poison");

                        await Task.Delay(3000);

                        Image.Source = LoadImage("Basket");
                    }
                }
                else
                {
                    Image.Source = LoadImage("Wi-FiOff");

                    await Task.Delay(3000);

                    Image.Source = LoadImage("Basket");
                }
            }
            else
            {
                State.Visibility = Visibility.Collapsed;
                Progress.Visibility = Visibility.Collapsed;
                PublishGrid.Visibility = Visibility.Collapsed;

                State.Text = SRER.GetValue("Portal", "ThemeShare", "ThemePublish", "Preparing");
            }

            Publish.IsEnabled = true;
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
        }

        private async void Publisher_Click(object sender, RoutedEventArgs e)
        {
            if (Category.SelectedIndex >= 0)
            {
                await Application.Current.Dispatcher.InvokeAsync(async () =>
                {
                    Publish.IsEnabled = false;
                    Category.IsEnabled = false;
                    Publisher.IsEnabled = false;

                    State.Visibility = Visibility.Visible;
                    Progress.Visibility = Visibility.Visible;

                    string TempFile = string.Empty;

                    using HttpClient Client = new();

                    HttpResponseMessage Response = new();

                    Client.DefaultRequestHeaders.Add("User-Agent", SMMG.UserAgent);

                    State.Text = SRER.GetValue("Portal", "ThemeShare", "ThemePublish", "Limit");

                    await Task.Delay(1000);

                    try
                    {
                        Response = await Client.GetAsync($"{SMMRU.Soferity}/{SMR.SoferityVersion}/{SMR.SoferityUpload}/{SMR.SoferityCheck}/{SSSHU.GetGuid()}");
                    }
                    catch
                    {
                        State.Text = SRER.GetValue("Portal", "ThemeShare", "ThemePublish", "Limit", "Error");
                    }

                    if (Response.IsSuccessStatusCode)
                    {
                        TempFile = Path.Combine(Path.GetTempPath(), $"{SSSHC.FileName(Info.Title)}.zip");

                        State.Text = SRER.GetValue("Portal", "ThemeShare", "ThemePublish", "Compress");

                        await Task.Delay(1000);

                        Response = new();

                        if (await Task.Run(() => SSZEZ.Compress(Theme, TempFile)) != SSDECT.Pass)
                        {
                            State.Text = SRER.GetValue("Portal", "ThemeShare", "ThemePublish", "Compress", "Error");
                        }
                        else
                        {
                            State.Text = SRER.GetValue("Portal", "ThemeShare", "ThemePublish", "Size");

                            await Task.Delay(1000);

                            FileInfo TempInfo = new(TempFile);

                            long TempSize = TempInfo.Length;
                            int LimitSize = 90;

                            if (TempSize > LimitSize * 1024 * 1024)
                            {
                                State.Text = string.Format(SRER.GetValue("Portal", "ThemeShare", "ThemePublish", "Size", "Exceeded"), LimitSize);
                            }
                            else
                            {
                                State.Text = SRER.GetValue("Portal", "ThemeShare", "ThemePublish", "Check");

                                await Task.Delay(1000);

                                if (await Task.Run(() => SSZHA.Check(TempFile)) != SSDECT.Pass)
                                {
                                    State.Text = SRER.GetValue("Portal", "ThemeShare", "ThemePublish", "Check", "Error");
                                }
                                else
                                {
                                    State.Text = SRER.GetValue("Portal", "ThemeShare", "ThemePublish", "Upload");

                                    await Task.Delay(1000);

                                    Progress.IsIndeterminate = false;

                                    using MultipartFormDataContent Content = new();
                                    using FileStream FileStream = new(TempFile, FileMode.Open, FileAccess.Read);
                                    using StreamContent FileContent = new(new SSSEPS(FileStream, TempSize, ReportProgress));

                                    FileContent.Headers.ContentType = MediaTypeHeaderValue.Parse("application/zip");
                                    Content.Add(FileContent, "file", Path.GetFileName(TempFile));

                                    State.Text = SRER.GetValue("Portal", "ThemeShare", "ThemePublish", "Upload", "Start");

                                    await Task.Delay(1000);

                                    try
                                    {
                                        Response = await Client.PostAsync($"{SMMRU.Soferity}/{SMR.SoferityVersion}/{SMR.SoferityUpload}/{SMR.SoferityTheme}/{SSSHU.GetGuid()}/{(Category.SelectedItem as ComboBoxItem).Tag}", Content);
                                    }
                                    catch
                                    {
                                        State.Text = SRER.GetValue("Portal", "ThemeShare", "ThemePublish", "Upload", "Error");
                                    }

                                    if (Response.IsSuccessStatusCode)
                                    {
                                        State.Text = SRER.GetValue("Portal", "ThemeShare", "ThemePublish", "Upload", "Succeded");
                                    }
                                    else
                                    {
                                        State.Text = string.Format(SRER.GetValue("Portal", "ThemeShare", "ThemePublish", "Upload", "Errored"), Response.StatusCode);
                                    }
                                }
                            }
                        }
                    }
                    else
                    {
                        State.Text = SRER.GetValue("Portal", "ThemeShare", "ThemePublish", "Limit", "Exceeded");
                    }

                    await Task.Delay(3000);

                    if (!string.IsNullOrEmpty(TempFile) && File.Exists(TempFile))
                    {
                        File.Delete(TempFile);
                    }

                    Publish.IsEnabled = true;
                    Category.IsEnabled = true;
                    Publisher.IsEnabled = true;

                    Progress.Value = 0;
                    Progress.IsIndeterminate = true;
                });
            }
        }

        private void ContentDialog_PreviewKeyDown(object sender, KeyEventArgs e)
        {
            if (e.Key is Key.Enter or Key.Escape)
            {
                e.Handled = true;
            }
        }

        private async void ReportProgress(long BytesTransferred, long TotalBytes, double Percentage)
        {
            await Application.Current.Dispatcher.InvokeAsync(() => Progress.Value = Percentage);
        }

        protected override void OnButtonClick(ContentDialogButton Button)
        {
            if (!Publisher.IsEnabled)
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