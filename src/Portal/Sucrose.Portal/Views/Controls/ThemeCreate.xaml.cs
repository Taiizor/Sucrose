using System.IO;
using System.Reflection;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using Wpf.Ui.Controls;
using Button = Wpf.Ui.Controls.Button;
using MessageBox = Wpf.Ui.Controls.MessageBox;
using OpenFileDialog = Microsoft.Win32.OpenFileDialog;
using SEAT = Skylark.Enum.AssemblyType;
using SHA = Skylark.Helper.Assemblies;
using SHG = Skylark.Helper.Generator;
using SMML = Sucrose.Manager.Manage.Library;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMR = Sucrose.Memory.Readonly;
using SPETC = Sucrose.Portal.Extension.ThemeCreate;
using SPETL = Sucrose.Portal.Extension.ThumbnailLoader;
using SPMI = Sucrose.Portal.Manage.Internal;
using SRER = Sucrose.Resources.Extension.Resources;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSSHA = Sucrose.Shared.Space.Helper.Access;
using SSSHC = Sucrose.Shared.Space.Helper.Copy;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeCreate.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeCreate : ContentDialog, IDisposable
    {
        private readonly SPETL Loader = new();

        public ThemeCreate(ContentPresenter? contentPresenter) : base(contentPresenter)
        {
            InitializeComponent();
        }

        private void Back_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = false;
            GifCard.Visibility = Visibility.Collapsed;
            UrlCard.Visibility = Visibility.Collapsed;
            WebCard.Visibility = Visibility.Collapsed;
            CreateCard.Visibility = Visibility.Visible;
            VideoCard.Visibility = Visibility.Collapsed;
            YouTubeCard.Visibility = Visibility.Collapsed;
            ApplicationCard.Visibility = Visibility.Collapsed;

            GifDelete_Click(null, null);
            VideoDelete_Click(null, null);

            Dispose();
        }

        private void GifArea_DragOver(object sender, DragEventArgs e)
        {
            if (!e.Data.GetDataPresent(DataFormats.FileDrop) || e.AllowedEffects.HasFlag(DragDropEffects.Copy) == false)
            {
                e.Effects = DragDropEffects.None;
            }
            else
            {
                e.Effects = DragDropEffects.Copy;
                GifRectangle.Stroke = Brushes.DodgerBlue;
            }
        }

        private void GifArea_DragLeave(object sender, DragEventArgs e)
        {
            if (string.IsNullOrEmpty($"{Loader.SourceUri}"))
            {
                GifRectangle.Stroke = SRER.GetResource<Brush>("TextFillColorDisabledBrush");
            }
            else
            {
                GifRectangle.Stroke = Brushes.SeaGreen;
            }
        }

        private void GifCreate_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = true;
            GifCard.Visibility = Visibility.Visible;
            CreateCard.Visibility = Visibility.Collapsed;

            GifAuthor.Text = SPETC.GetAuthor();
            GifContact.Text = SPETC.GetContact();
        }

        private void UrlCreate_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = true;
            UrlCard.Visibility = Visibility.Visible;
            CreateCard.Visibility = Visibility.Collapsed;

            UrlAuthor.Text = SPETC.GetAuthor();
            UrlContact.Text = SPETC.GetContact();
        }

        private void WebCreate_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = true;
            WebCard.Visibility = Visibility.Visible;
            CreateCard.Visibility = Visibility.Collapsed;

            WebAuthor.Text = SPETC.GetAuthor();
            WebContact.Text = SPETC.GetContact();
        }

        private void GifDelete_Click(object sender, RoutedEventArgs e)
        {
            Loader.Dispose();
            Loader.SourceUri = null;
            Loader.SourcePath = null;
            GifImagine.Source = null;
            GifIcon.Visibility = Visibility.Visible;
            GifText.Visibility = Visibility.Visible;
            GifDelete.Visibility = Visibility.Collapsed;
            GifRectangle.Stroke = SRER.GetResource<Brush>("TextFillColorDisabledBrush");
        }

        private void VideoArea_DragOver(object sender, DragEventArgs e)
        {
            if (!e.Data.GetDataPresent(DataFormats.FileDrop) || e.AllowedEffects.HasFlag(DragDropEffects.Copy) == false)
            {
                e.Effects = DragDropEffects.None;
            }
            else
            {
                e.Effects = DragDropEffects.Copy;
                VideoRectangle.Stroke = Brushes.DodgerBlue;
            }
        }

        private async void GifArea_Drop(object sender, DragEventArgs e)
        {
            GifRectangle.Stroke = SRER.GetResource<Brush>("TextFillColorDisabledBrush");

            if (e.Data.GetDataPresent(DataFormats.FileDrop))
            {
                string[] Files = (string[])e.Data.GetData(DataFormats.FileDrop);

                if (Files.Any())
                {
                    foreach (string Record in Files)
                    {
                        if (SSSHA.File(Record))
                        {
                            string Extension = Path.GetExtension(Record).ToLowerInvariant();

                            if (Extension is ".gif")
                            {
                                GifDescription.Text = SPETC.GetDescription(Path.GetFileNameWithoutExtension(Record), SSDEWT.Gif);
                                GifTitle.Text = SPETC.GetTitle(Path.GetFileNameWithoutExtension(Record));
                                GifImagine.Source = await Loader.LoadAsync(Record);
                                GifDelete.Visibility = Visibility.Visible;
                                GifIcon.Visibility = Visibility.Collapsed;
                                GifText.Visibility = Visibility.Collapsed;
                                GifRectangle.Stroke = Brushes.SeaGreen;
                                break;
                            }
                        }
                        else
                        {
                            MessageBox Warning = new()
                            {
                                Title = SRER.GetValue("Portal", "ThemeCreate", "Access", "Title"),
                                Content = SRER.GetValue("Portal", "ThemeCreate", "Access", "Message"),
                                CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Access", "Close")
                            };

                            await Warning.ShowDialogAsync();
                        }
                    }
                }
            }
        }

        private void VideoDelete_Click(object sender, RoutedEventArgs e)
        {
            Loader.Dispose();
            Loader.SourceUri = null;
            Loader.SourcePath = null;
            VideoImagine.Source = null;
            VideoIcon.Visibility = Visibility.Visible;
            VideoText.Visibility = Visibility.Visible;
            VideoDelete.Visibility = Visibility.Collapsed;
            VideoRectangle.Stroke = SRER.GetResource<Brush>("TextFillColorDisabledBrush");
        }

        private void VideoCreate_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = true;
            VideoCard.Visibility = Visibility.Visible;
            CreateCard.Visibility = Visibility.Collapsed;

            VideoAuthor.Text = SPETC.GetAuthor();
            VideoContact.Text = SPETC.GetContact();
        }

        private void VideoArea_DragLeave(object sender, DragEventArgs e)
        {
            if (string.IsNullOrEmpty($"{Loader.SourceUri}"))
            {
                VideoRectangle.Stroke = SRER.GetResource<Brush>("TextFillColorDisabledBrush");
            }
            else
            {
                VideoRectangle.Stroke = Brushes.SeaGreen;
            }
        }

        private async void VideoArea_Drop(object sender, DragEventArgs e)
        {
            VideoRectangle.Stroke = SRER.GetResource<Brush>("TextFillColorDisabledBrush");

            if (e.Data.GetDataPresent(DataFormats.FileDrop))
            {
                string[] Files = (string[])e.Data.GetData(DataFormats.FileDrop);

                if (Files.Any())
                {
                    foreach (string Record in Files)
                    {
                        if (SSSHA.File(Record))
                        {
                            string Extension = Path.GetExtension(Record).ToLowerInvariant();

                            if (Extension is ".mp4" or ".avi" or ".mov" or ".mkv" or ".ogv" or ".flv" or ".wmv" or ".hevc" or ".webm" or ".mpeg" or ".mpeg1" or ".mpeg2" or ".mpeg4")
                            {
                                VideoDescription.Text = SPETC.GetDescription(Path.GetFileNameWithoutExtension(Record), SSDEWT.Video);
                                VideoTitle.Text = SPETC.GetTitle(Path.GetFileNameWithoutExtension(Record));
                                VideoImagine.Source = await Loader.LoadAsync(Record);
                                VideoDelete.Visibility = Visibility.Visible;
                                VideoIcon.Visibility = Visibility.Collapsed;
                                VideoText.Visibility = Visibility.Collapsed;
                                VideoRectangle.Stroke = Brushes.SeaGreen;
                                break;
                            }
                        }
                        else
                        {
                            MessageBox Warning = new()
                            {
                                Title = SRER.GetValue("Portal", "ThemeCreate", "Access", "Title"),
                                Content = SRER.GetValue("Portal", "ThemeCreate", "Access", "Message"),
                                CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Access", "Close")
                            };

                            await Warning.ShowDialogAsync();
                        }
                    }
                }
            }
        }

        private void YouTubeCreate_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = true;
            YouTubeCard.Visibility = Visibility.Visible;
            CreateCard.Visibility = Visibility.Collapsed;

            YouTubeAuthor.Text = SPETC.GetAuthor();
            YouTubeContact.Text = SPETC.GetContact();
        }

        private void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            GifExpander.TitleText = SRER.GetValue("Portal", "ThemeCreate", "Gif");
            GifExpander.DescriptionText = SRER.GetValue("Portal", "ThemeCreate", "Gif", "Description");

            UrlExpander.TitleText = SRER.GetValue("Portal", "ThemeCreate", "Url");
            UrlExpander.DescriptionText = SRER.GetValue("Portal", "ThemeCreate", "Url", "Description");

            WebExpander.TitleText = SRER.GetValue("Portal", "ThemeCreate", "Web");
            WebExpander.DescriptionText = SRER.GetValue("Portal", "ThemeCreate", "Web", "Description");

            VideoExpander.TitleText = SRER.GetValue("Portal", "ThemeCreate", "Video");
            VideoExpander.DescriptionText = SRER.GetValue("Portal", "ThemeCreate", "Video", "Description");

            YouTubeExpander.TitleText = SRER.GetValue("Portal", "ThemeCreate", "YouTube");
            YouTubeExpander.DescriptionText = SRER.GetValue("Portal", "ThemeCreate", "YouTube", "Description");

            ApplicationExpander.TitleText = SRER.GetValue("Portal", "ThemeCreate", "Application");
            ApplicationExpander.DescriptionText = SRER.GetValue("Portal", "ThemeCreate", "Application", "Description");
        }

        private void WebSourceClear_Click(object sender, RoutedEventArgs e)
        {
            WebSource.Content = SRER.GetValue("Portal", "ThemeCreate", "ThemeWeb", "Hint");
        }

        private async void WebSource_Click(object sender, RoutedEventArgs e)
        {
            Button Button = sender as Button;

            string Startup = File.Exists($"{Button.Content}") ? Path.GetDirectoryName($"{Button.Content}") : SMMRP.Desktop;

            OpenFileDialog FileDialog = new()
            {
                Filter = SRER.GetValue("Portal", "ThemeCreate", "WebSource", "Filter"),
                FilterIndex = 1,

                Title = SRER.GetValue("Portal", "ThemeCreate", "WebSource", "Title"),

                InitialDirectory = Startup
            };

            if (FileDialog.ShowDialog() == true)
            {
                if (SSSHA.File(FileDialog.FileName))
                {
                    Button.Content = FileDialog.FileName;
                    Button.BorderBrush = WebThumbnail.BorderBrush;
                    WebTitle.Text = SPETC.GetTitle(Path.GetFileNameWithoutExtension(FileDialog.FileName));
                    WebDescription.Text = SPETC.GetDescription(Path.GetFileNameWithoutExtension(FileDialog.FileName), SSDEWT.Web);

                    MessageBox Warning = new()
                    {
                        Title = SRER.GetValue("Portal", "ThemeCreate", "Copy", "Title"),
                        Content = SRER.GetValue("Portal", "ThemeCreate", "Copy", "Message"),
                        CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Copy", "Close")
                    };

                    await Warning.ShowDialogAsync();
                }
                else
                {
                    MessageBox Warning = new()
                    {
                        Title = SRER.GetValue("Portal", "ThemeCreate", "Access", "Title"),
                        Content = SRER.GetValue("Portal", "ThemeCreate", "Access", "Message"),
                        CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Access", "Close")
                    };

                    await Warning.ShowDialogAsync();
                }
            }
        }

        private void ApplicationCreate_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = true;
            CreateCard.Visibility = Visibility.Collapsed;
            ApplicationCard.Visibility = Visibility.Visible;

            ApplicationAuthor.Text = SPETC.GetAuthor();
            ApplicationContact.Text = SPETC.GetContact();
        }

        private async void ThemePreview_Click(object sender, RoutedEventArgs e)
        {
            Button Button = sender as Button;

            string Startup = File.Exists($"{Button.Content}") ? Path.GetDirectoryName($"{Button.Content}") : SMMRP.Desktop;

            OpenFileDialog FileDialog = new()
            {
                Filter = SRER.GetValue("Portal", "ThemeCreate", "ThemePreview", "Filter"),
                FilterIndex = 1,

                Title = SRER.GetValue("Portal", "ThemeCreate", "ThemePreview", "Title"),

                InitialDirectory = Startup
            };

            if (FileDialog.ShowDialog() == true)
            {
                if (SSSHA.File(FileDialog.FileName))
                {
                    Button.Content = FileDialog.FileName;
                }
                else
                {
                    MessageBox Warning = new()
                    {
                        Title = SRER.GetValue("Portal", "ThemeCreate", "Access", "Title"),
                        Content = SRER.GetValue("Portal", "ThemeCreate", "Access", "Message"),
                        CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Access", "Close")
                    };

                    await Warning.ShowDialogAsync();
                }
            }
        }

        private void ContentDialog_PreviewKeyDown(object sender, KeyEventArgs e)
        {
            if (e.Key is Key.Enter or Key.Escape)
            {
                e.Handled = true;
            }
        }

        private async void ThemeThumbnail_Click(object sender, RoutedEventArgs e)
        {
            Button Button = sender as Button;

            string Startup = File.Exists($"{Button.Content}") ? Path.GetDirectoryName($"{Button.Content}") : SMMRP.Desktop;

            OpenFileDialog FileDialog = new()
            {
                Filter = SRER.GetValue("Portal", "ThemeCreate", "ThemeThumbnail", "Filter"),
                FilterIndex = 1,

                Title = SRER.GetValue("Portal", "ThemeCreate", "ThemeThumbnail", "Title"),

                InitialDirectory = Startup
            };

            if (FileDialog.ShowDialog() == true)
            {
                if (SSSHA.File(FileDialog.FileName))
                {
                    Button.Content = FileDialog.FileName;
                }
                else
                {
                    MessageBox Warning = new()
                    {
                        Title = SRER.GetValue("Portal", "ThemeCreate", "Access", "Title"),
                        Content = SRER.GetValue("Portal", "ThemeCreate", "Access", "Message"),
                        CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Access", "Close")
                    };

                    await Warning.ShowDialogAsync();
                }
            }
        }

        private void ApplicationSourceClear_Click(object sender, RoutedEventArgs e)
        {
            ApplicationSource.Content = SRER.GetValue("Portal", "ThemeCreate", "ThemeApplication", "Hint");
        }

        private static async Task ExtractResources(string Source, string ExtractPath)
        {
            Assembly Entry = SHA.Assemble(SEAT.Entry);
            string[] Resources = Entry.GetManifestResourceNames();

            foreach (string Resource in Resources)
            {
                if (Resource.EndsWith(Source))
                {
                    string ExtractFilePath = Path.Combine(ExtractPath, Source);

                    if (!Directory.Exists(Path.GetDirectoryName(ExtractFilePath)))
                    {
                        Directory.CreateDirectory(Path.GetDirectoryName(ExtractFilePath));
                    }

                    using Stream ResourceStream = Entry.GetManifestResourceStream(Resource);
                    using FileStream OutputFileStream = new(ExtractFilePath, FileMode.OpenOrCreate);

                    await ResourceStream.CopyToAsync(OutputFileStream);
                }
            }
        }

        private async void ApplicationSource_Click(object sender, RoutedEventArgs e)
        {
            Button Button = sender as Button;

            string Startup = File.Exists($"{Button.Content}") ? Path.GetDirectoryName($"{Button.Content}") : SMMRP.Desktop;

            OpenFileDialog FileDialog = new()
            {
                Filter = SRER.GetValue("Portal", "ThemeCreate", "ApplicationSource", "Filter"),
                FilterIndex = 1,

                Title = SRER.GetValue("Portal", "ThemeCreate", "ApplicationSource", "Title"),

                InitialDirectory = Startup
            };

            if (FileDialog.ShowDialog() == true)
            {
                if (SSSHA.File(FileDialog.FileName))
                {
                    Button.Content = FileDialog.FileName;
                    Button.BorderBrush = ApplicationThumbnail.BorderBrush;
                    ApplicationTitle.Text = SPETC.GetTitle(Path.GetFileNameWithoutExtension(FileDialog.FileName));
                    ApplicationDescription.Text = SPETC.GetDescription(Path.GetFileNameWithoutExtension(FileDialog.FileName), SSDEWT.Application);

                    MessageBox Warning = new()
                    {
                        Title = SRER.GetValue("Portal", "ThemeCreate", "Copy", "Title"),
                        Content = SRER.GetValue("Portal", "ThemeCreate", "Copy", "Message"),
                        CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Copy", "Close")
                    };

                    await Warning.ShowDialogAsync();
                }
                else
                {
                    MessageBox Warning = new()
                    {
                        Title = SRER.GetValue("Portal", "ThemeCreate", "Access", "Title"),
                        Content = SRER.GetValue("Portal", "ThemeCreate", "Access", "Message"),
                        CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Access", "Close")
                    };

                    await Warning.ShowDialogAsync();
                }
            }
        }

        private async void GifArea_MouseLeftButtonUp(object sender, MouseButtonEventArgs e)
        {
            string Startup = Loader != null && !string.IsNullOrEmpty($"{Loader.SourceUri}") && File.Exists(Loader.SourceUri.LocalPath) ? Path.GetDirectoryName(Loader.SourceUri.LocalPath) : SMMRP.Desktop;

            OpenFileDialog FileDialog = new()
            {
                Filter = SRER.GetValue("Portal", "ThemeCreate", "DragDrop", "Gif", "Filter"),
                FilterIndex = 1,

                Title = SRER.GetValue("Portal", "ThemeCreate", "DragDrop", "Gif", "Title"),

                InitialDirectory = Startup
            };

            if (FileDialog.ShowDialog() == true)
            {
                if (SSSHA.File(FileDialog.FileName))
                {
                    GifDescription.Text = SPETC.GetDescription(Path.GetFileNameWithoutExtension(FileDialog.FileName), SSDEWT.Gif);
                    GifTitle.Text = SPETC.GetTitle(Path.GetFileNameWithoutExtension(FileDialog.FileName));
                    GifImagine.Source = await Loader.LoadAsync(FileDialog.FileName);
                    GifDelete.Visibility = Visibility.Visible;
                    GifIcon.Visibility = Visibility.Collapsed;
                    GifText.Visibility = Visibility.Collapsed;
                    GifRectangle.Stroke = Brushes.SeaGreen;
                }
                else
                {
                    MessageBox Warning = new()
                    {
                        Title = SRER.GetValue("Portal", "ThemeCreate", "Access", "Title"),
                        Content = SRER.GetValue("Portal", "ThemeCreate", "Access", "Message"),
                        CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Access", "Close")
                    };

                    await Warning.ShowDialogAsync();
                }
            }
        }

        private async void VideoArea_MouseLeftButtonUp(object sender, MouseButtonEventArgs e)
        {
            string Startup = Loader != null && !string.IsNullOrEmpty($"{Loader.SourceUri}") && File.Exists(Loader.SourceUri.LocalPath) ? Path.GetDirectoryName(Loader.SourceUri.LocalPath) : SMMRP.Desktop;

            OpenFileDialog FileDialog = new()
            {
                Filter = SRER.GetValue("Portal", "ThemeCreate", "DragDrop", "Video", "Filter"),
                FilterIndex = 1,

                Title = SRER.GetValue("Portal", "ThemeCreate", "DragDrop", "Video", "Title"),

                InitialDirectory = Startup
            };

            if (FileDialog.ShowDialog() == true)
            {
                if (SSSHA.File(FileDialog.FileName))
                {
                    VideoDescription.Text = SPETC.GetDescription(Path.GetFileNameWithoutExtension(FileDialog.FileName), SSDEWT.Video);
                    VideoTitle.Text = SPETC.GetTitle(Path.GetFileNameWithoutExtension(FileDialog.FileName));
                    VideoImagine.Source = await Loader.LoadAsync(FileDialog.FileName);
                    VideoDelete.Visibility = Visibility.Visible;
                    VideoIcon.Visibility = Visibility.Collapsed;
                    VideoText.Visibility = Visibility.Collapsed;
                    VideoRectangle.Stroke = Brushes.SeaGreen;
                }
                else
                {
                    MessageBox Warning = new()
                    {
                        Title = SRER.GetValue("Portal", "ThemeCreate", "Access", "Title"),
                        Content = SRER.GetValue("Portal", "ThemeCreate", "Access", "Message"),
                        CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Access", "Close")
                    };

                    await Warning.ShowDialogAsync();
                }
            }
        }

        protected override async void OnButtonClick(ContentDialogButton Button)
        {
            if (Button == ContentDialogButton.Primary)
            {
                if (!Directory.Exists(SMML.LibraryLocation))
                {
                    Directory.CreateDirectory(SMML.LibraryLocation);
                }

                if (GifCard.Visibility == Visibility.Visible)
                {
                    Uri Gif = Loader.SourceUri;

                    if (Gif == null || string.IsNullOrEmpty(Gif.LocalPath))
                    {
                        GifRectangle.Stroke = Brushes.Crimson;
                        return;
                    }
                    else if (string.IsNullOrEmpty(GifTitle.Text))
                    {
                        GifTitle.Focus();
                        return;
                    }
                    else if (string.IsNullOrEmpty(GifDescription.Text))
                    {
                        GifDescription.Focus();
                        return;
                    }
                    else if (string.IsNullOrEmpty(GifAuthor.Text))
                    {
                        GifAuthor.Focus();
                        return;
                    }
                    else if (!SSTHV.IsUrl(GifContact.Text) && !SSTHV.IsMail(GifContact.Text))
                    {
                        GifContact.Focus();
                        return;
                    }
                    else
                    {
                        string Name;
                        string Theme;
                        string[] Tags;
                        string Preview = "Preview.gif";
                        string Thumbnail = "Thumbnail.jpg";

                        if (string.IsNullOrEmpty(GifTags.Text))
                        {
                            Tags = Array.Empty<string>();
                        }
                        else
                        {
                            if (GifTags.Text.Contains(','))
                            {
                                Tags = GifTags.Text.Split(',').Select(Tag => Tag.TrimStart().TrimEnd()).ToArray();

                                if (Tags.Count() is < 1 or > 5)
                                {
                                    GifTags.Focus();
                                    return;
                                }
                                else if (Tags.Any(Tag => Tag.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(Tag)))
                                {
                                    GifTags.Focus();
                                    return;
                                }
                            }
                            else
                            {
                                if (GifTags.Text.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(GifTags.Text))
                                {
                                    GifTags.Focus();
                                    return;
                                }
                                else
                                {
                                    Tags = new[]
                                    {
                                        GifTags.Text.TrimStart().TrimEnd()
                                    };
                                }
                            }
                        }

                        do
                        {
                            SPMI.LibraryService.Theme = SHG.GenerateString(SMMM.Chars, 25, SMR.Randomise);
                            Theme = Path.Combine(SMML.LibraryLocation, SPMI.LibraryService.Theme);
                        } while (Directory.Exists(Theme));

                        Directory.CreateDirectory(Theme);

                        if (File.Exists(Gif.LocalPath))
                        {
                            Name = "s_" + Path.GetFileName(Gif.LocalPath);
                            await Task.Run(() => File.Copy(Gif.LocalPath, Path.Combine(Theme, Name), true));
                        }
                        else
                        {
                            GifRectangle.Stroke = Brushes.Crimson;
                            return;
                        }

                        if (File.Exists($"{GifThumbnail.Content}"))
                        {
                            string Source = $"{GifThumbnail.Content}";
                            Thumbnail = "t_" + Path.GetFileName(Source);
                            await Task.Run(() => File.Copy(Source, Path.Combine(Theme, Thumbnail), true));
                        }
                        else
                        {
                            try
                            {
                                using FileStream Stream = new(Path.Combine(Theme, Thumbnail), FileMode.Create);

                                BitmapEncoder Encoder = new JpegBitmapEncoder();

                                Encoder.Frames.Add(BitmapFrame.Create((BitmapSource)GifImagine.Source));

                                Encoder.Save(Stream);
                            }
                            catch
                            {
                                await Task.Run(async () => await ExtractResources(Thumbnail, Theme));
                            }
                        }

                        if (File.Exists($"{GifPreview.Content}"))
                        {
                            string Source = $"{GifPreview.Content}";
                            Preview = "p_" + Path.GetFileName(Source);
                            await Task.Run(() => File.Copy(Source, Path.Combine(Theme, Preview), true));
                        }
                        else
                        {
                            await Task.Run(async () => await ExtractResources(Preview, Theme));
                        }

                        SSTHI Info = new()
                        {
                            Tags = Tags,
                            Source = Name,
                            License = null,
                            Arguments = null,
                            Type = SSDEWT.Gif,
                            Preview = Preview,
                            Thumbnail = Thumbnail,
                            Title = GifTitle.Text,
                            Author = GifAuthor.Text,
                            AppVersion = SSCHV.Get(),
                            Version = new(1, 0, 0, 0),
                            Contact = GifContact.Text,
                            Description = GifDescription.Text
                        };

                        SSTHI.WriteJson(Path.Combine(Theme, SMR.SucroseInfo), Info);
                    }
                }
                else if (UrlCard.Visibility == Visibility.Visible)
                {
                    if (!SSTHV.IsUrl(UrlUrl.Text))
                    {
                        UrlUrl.Focus();
                        return;
                    }
                    else if (string.IsNullOrEmpty(UrlTitle.Text))
                    {
                        UrlTitle.Focus();
                        return;
                    }
                    else if (string.IsNullOrEmpty(UrlDescription.Text))
                    {
                        UrlDescription.Focus();
                        return;
                    }
                    else if (string.IsNullOrEmpty(UrlAuthor.Text))
                    {
                        UrlAuthor.Focus();
                        return;
                    }
                    else if (!SSTHV.IsUrl(UrlContact.Text) && !SSTHV.IsMail(UrlContact.Text))
                    {
                        UrlContact.Focus();
                        return;
                    }
                    else
                    {
                        string Theme;
                        string[] Tags;
                        string Preview = "Preview.gif";
                        string Thumbnail = "Thumbnail.jpg";

                        if (string.IsNullOrEmpty(UrlTags.Text))
                        {
                            Tags = Array.Empty<string>();
                        }
                        else
                        {
                            if (UrlTags.Text.Contains(','))
                            {
                                Tags = UrlTags.Text.Split(',').Select(Tag => Tag.TrimStart().TrimEnd()).ToArray();

                                if (Tags.Count() is < 1 or > 5)
                                {
                                    UrlTags.Focus();
                                    return;
                                }
                                else if (Tags.Any(Tag => Tag.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(Tag)))
                                {
                                    UrlTags.Focus();
                                    return;
                                }
                            }
                            else
                            {
                                if (UrlTags.Text.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(UrlTags.Text))
                                {
                                    UrlTags.Focus();
                                    return;
                                }
                                else
                                {
                                    Tags = new[]
                                    {
                                        UrlTags.Text.TrimStart().TrimEnd()
                                    };
                                }
                            }
                        }

                        do
                        {
                            SPMI.LibraryService.Theme = SHG.GenerateString(SMMM.Chars, 25, SMR.Randomise);
                            Theme = Path.Combine(SMML.LibraryLocation, SPMI.LibraryService.Theme);
                        } while (Directory.Exists(Theme));

                        Directory.CreateDirectory(Theme);

                        if (File.Exists($"{UrlThumbnail.Content}"))
                        {
                            string Source = $"{UrlThumbnail.Content}";
                            Thumbnail = "t_" + Path.GetFileName(Source);
                            await Task.Run(() => File.Copy(Source, Path.Combine(Theme, Thumbnail), true));
                        }
                        else
                        {
                            await Task.Run(async () => await ExtractResources(Thumbnail, Theme));
                        }

                        if (File.Exists($"{UrlPreview.Content}"))
                        {
                            string Source = $"{UrlPreview.Content}";
                            Preview = "p_" + Path.GetFileName(Source);
                            await Task.Run(() => File.Copy(Source, Path.Combine(Theme, Preview), true));
                        }
                        else
                        {
                            await Task.Run(async () => await ExtractResources(Preview, Theme));
                        }

                        SSTHI Info = new()
                        {
                            Tags = Tags,
                            License = null,
                            Arguments = null,
                            Type = SSDEWT.Url,
                            Preview = Preview,
                            Source = UrlUrl.Text,
                            Thumbnail = Thumbnail,
                            Title = UrlTitle.Text,
                            Author = UrlAuthor.Text,
                            AppVersion = SSCHV.Get(),
                            Version = new(1, 0, 0, 0),
                            Contact = UrlContact.Text,
                            Description = UrlDescription.Text
                        };

                        SSTHI.WriteJson(Path.Combine(Theme, SMR.SucroseInfo), Info);
                    }
                }
                else if (WebCard.Visibility == Visibility.Visible)
                {
                    if (Path.GetExtension($"{WebSource.Content}") != ".html")
                    {
                        WebSource.BorderBrush = Brushes.Crimson;
                        WebSource.Focus();
                        return;
                    }
                    else if (string.IsNullOrEmpty(WebTitle.Text))
                    {
                        WebTitle.Focus();
                        return;
                    }
                    else if (string.IsNullOrEmpty(WebDescription.Text))
                    {
                        WebDescription.Focus();
                        return;
                    }
                    else if (string.IsNullOrEmpty(WebAuthor.Text))
                    {
                        WebAuthor.Focus();
                        return;
                    }
                    else if (!SSTHV.IsUrl(WebContact.Text) && !SSTHV.IsMail(WebContact.Text))
                    {
                        WebContact.Focus();
                        return;
                    }
                    else
                    {
                        string Theme;
                        string[] Tags;
                        string Preview = "Preview.gif";
                        string Thumbnail = "Thumbnail.jpg";

                        if (string.IsNullOrEmpty(WebTags.Text))
                        {
                            Tags = Array.Empty<string>();
                        }
                        else
                        {
                            if (WebTags.Text.Contains(','))
                            {
                                Tags = WebTags.Text.Split(',').Select(Tag => Tag.TrimStart().TrimEnd()).ToArray();

                                if (Tags.Count() is < 1 or > 5)
                                {
                                    WebTags.Focus();
                                    return;
                                }
                                else if (Tags.Any(Tag => Tag.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(Tag)))
                                {
                                    WebTags.Focus();
                                    return;
                                }
                            }
                            else
                            {
                                if (WebTags.Text.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(WebTags.Text))
                                {
                                    WebTags.Focus();
                                    return;
                                }
                                else
                                {
                                    Tags = new[]
                                    {
                                        WebTags.Text.TrimStart().TrimEnd()
                                    };
                                }
                            }
                        }

                        do
                        {
                            SPMI.LibraryService.Theme = SHG.GenerateString(SMMM.Chars, 25, SMR.Randomise);
                            Theme = Path.Combine(SMML.LibraryLocation, SPMI.LibraryService.Theme);
                        } while (Directory.Exists(Theme));

                        Directory.CreateDirectory(Theme);

                        SSSHC.Folder(Path.GetDirectoryName($"{WebSource.Content}"), Theme, false);

                        if (File.Exists($"{WebThumbnail.Content}"))
                        {
                            string Source = $"{WebThumbnail.Content}";
                            Thumbnail = "t_" + Path.GetFileName(Source);
                            await Task.Run(() => File.Copy(Source, Path.Combine(Theme, Thumbnail), true));
                        }
                        else
                        {
                            await Task.Run(async () => await ExtractResources(Thumbnail, Theme));
                        }

                        if (File.Exists($"{WebPreview.Content}"))
                        {
                            string Source = $"{WebPreview.Content}";
                            Preview = "p_" + Path.GetFileName(Source);
                            await Task.Run(() => File.Copy(Source, Path.Combine(Theme, Preview), true));
                        }
                        else
                        {
                            await Task.Run(async () => await ExtractResources(Preview, Theme));
                        }

                        SSTHI Info = new()
                        {
                            Tags = Tags,
                            License = null,
                            Arguments = null,
                            Type = SSDEWT.Web,
                            Preview = Preview,
                            Thumbnail = Thumbnail,
                            Title = WebTitle.Text,
                            Author = WebAuthor.Text,
                            AppVersion = SSCHV.Get(),
                            Version = new(1, 0, 0, 0),
                            Contact = WebContact.Text,
                            Description = WebDescription.Text,
                            Source = Path.GetFileName($"{WebSource.Content}")
                        };

                        SSTHI.WriteJson(Path.Combine(Theme, SMR.SucroseInfo), Info);
                    }
                }
                else if (VideoCard.Visibility == Visibility.Visible)
                {
                    Uri Video = Loader.SourceUri;

                    if (Video == null || string.IsNullOrEmpty(Video.LocalPath))
                    {
                        VideoRectangle.Stroke = Brushes.Crimson;
                        return;
                    }
                    else if (string.IsNullOrEmpty(VideoTitle.Text))
                    {
                        VideoTitle.Focus();
                        return;
                    }
                    else if (string.IsNullOrEmpty(VideoDescription.Text))
                    {
                        VideoDescription.Focus();
                        return;
                    }
                    else if (string.IsNullOrEmpty(VideoAuthor.Text))
                    {
                        VideoAuthor.Focus();
                        return;
                    }
                    else if (!SSTHV.IsUrl(VideoContact.Text) && !SSTHV.IsMail(VideoContact.Text))
                    {
                        VideoContact.Focus();
                        return;
                    }
                    else
                    {
                        string Name;
                        string Theme;
                        string[] Tags;
                        string Preview = "Preview.gif";
                        string Thumbnail = "Thumbnail.jpg";

                        if (string.IsNullOrEmpty(VideoTags.Text))
                        {
                            Tags = Array.Empty<string>();
                        }
                        else
                        {
                            if (VideoTags.Text.Contains(','))
                            {
                                Tags = VideoTags.Text.Split(',').Select(Tag => Tag.TrimStart().TrimEnd()).ToArray();

                                if (Tags.Count() is < 1 or > 5)
                                {
                                    VideoTags.Focus();
                                    return;
                                }
                                else if (Tags.Any(Tag => Tag.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(Tag)))
                                {
                                    VideoTags.Focus();
                                    return;
                                }
                            }
                            else
                            {
                                if (VideoTags.Text.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(VideoTags.Text))
                                {
                                    VideoTags.Focus();
                                    return;
                                }
                                else
                                {
                                    Tags = new[]
                                    {
                                        VideoTags.Text.TrimStart().TrimEnd()
                                    };
                                }
                            }
                        }

                        do
                        {
                            SPMI.LibraryService.Theme = SHG.GenerateString(SMMM.Chars, 25, SMR.Randomise);
                            Theme = Path.Combine(SMML.LibraryLocation, SPMI.LibraryService.Theme);
                        } while (Directory.Exists(Theme));

                        Directory.CreateDirectory(Theme);

                        if (File.Exists(Video.LocalPath))
                        {
                            Name = "s_" + Path.GetFileName(Video.LocalPath);
                            await Task.Run(() => File.Copy(Video.LocalPath, Path.Combine(Theme, Name), true));
                        }
                        else
                        {
                            VideoRectangle.Stroke = Brushes.Crimson;
                            return;
                        }

                        if (File.Exists($"{VideoThumbnail.Content}"))
                        {
                            string Source = $"{VideoThumbnail.Content}";
                            Thumbnail = "t_" + Path.GetFileName(Source);
                            await Task.Run(() => File.Copy(Source, Path.Combine(Theme, Thumbnail), true));
                        }
                        else
                        {
                            try
                            {
                                using FileStream Stream = new(Path.Combine(Theme, Thumbnail), FileMode.Create);

                                BitmapEncoder Encoder = new JpegBitmapEncoder();

                                Encoder.Frames.Add(BitmapFrame.Create((BitmapSource)VideoImagine.Source));

                                Encoder.Save(Stream);
                            }
                            catch
                            {
                                await Task.Run(async () => await ExtractResources(Thumbnail, Theme));
                            }
                        }

                        if (File.Exists($"{VideoPreview.Content}"))
                        {
                            string Source = $"{VideoPreview.Content}";
                            Preview = "p_" + Path.GetFileName(Source);
                            await Task.Run(() => File.Copy(Source, Path.Combine(Theme, Preview), true));
                        }
                        else
                        {
                            await Task.Run(async () => await ExtractResources(Preview, Theme));
                        }

                        SSTHI Info = new()
                        {
                            Tags = Tags,
                            Source = Name,
                            License = null,
                            Arguments = null,
                            Preview = Preview,
                            Type = SSDEWT.Video,
                            Thumbnail = Thumbnail,
                            Title = VideoTitle.Text,
                            AppVersion = SSCHV.Get(),
                            Version = new(1, 0, 0, 0),
                            Author = VideoAuthor.Text,
                            Contact = VideoContact.Text,
                            Description = VideoDescription.Text
                        };

                        SSTHI.WriteJson(Path.Combine(Theme, SMR.SucroseInfo), Info);
                    }
                }
                else if (YouTubeCard.Visibility == Visibility.Visible)
                {
                    if (!SSTHV.IsYouTubeAll(YouTubeUrl.Text))
                    {
                        YouTubeUrl.Focus();
                        return;
                    }
                    else if (string.IsNullOrEmpty(YouTubeTitle.Text))
                    {
                        YouTubeTitle.Focus();
                        return;
                    }
                    else if (string.IsNullOrEmpty(YouTubeDescription.Text))
                    {
                        YouTubeDescription.Focus();
                        return;
                    }
                    else if (string.IsNullOrEmpty(YouTubeAuthor.Text))
                    {
                        YouTubeAuthor.Focus();
                        return;
                    }
                    else if (!SSTHV.IsUrl(YouTubeContact.Text) && !SSTHV.IsMail(YouTubeContact.Text))
                    {
                        YouTubeContact.Focus();
                        return;
                    }
                    else
                    {
                        string Theme;
                        string[] Tags;
                        string Preview = "Preview.gif";
                        string Thumbnail = "Thumbnail.jpg";

                        if (string.IsNullOrEmpty(YouTubeTags.Text))
                        {
                            Tags = Array.Empty<string>();
                        }
                        else
                        {
                            if (YouTubeTags.Text.Contains(','))
                            {
                                Tags = YouTubeTags.Text.Split(',').Select(Tag => Tag.TrimStart().TrimEnd()).ToArray();

                                if (Tags.Count() is < 1 or > 5)
                                {
                                    YouTubeTags.Focus();
                                    return;
                                }
                                else if (Tags.Any(Tag => Tag.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(Tag)))
                                {
                                    YouTubeTags.Focus();
                                    return;
                                }
                            }
                            else
                            {
                                if (YouTubeTags.Text.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(YouTubeTags.Text))
                                {
                                    YouTubeTags.Focus();
                                    return;
                                }
                                else
                                {
                                    Tags = new[]
                                    {
                                        YouTubeTags.Text.TrimStart().TrimEnd()
                                    };
                                }
                            }
                        }

                        do
                        {
                            SPMI.LibraryService.Theme = SHG.GenerateString(SMMM.Chars, 25, SMR.Randomise);
                            Theme = Path.Combine(SMML.LibraryLocation, SPMI.LibraryService.Theme);
                        } while (Directory.Exists(Theme));

                        Directory.CreateDirectory(Theme);

                        if (File.Exists($"{YouTubeThumbnail.Content}"))
                        {
                            string Source = $"{YouTubeThumbnail.Content}";
                            Thumbnail = "t_" + Path.GetFileName(Source);
                            await Task.Run(() => File.Copy(Source, Path.Combine(Theme, Thumbnail), true));
                        }
                        else
                        {
                            await Task.Run(async () => await ExtractResources(Thumbnail, Theme));
                        }

                        if (File.Exists($"{YouTubePreview.Content}"))
                        {
                            string Source = $"{YouTubePreview.Content}";
                            Preview = "p_" + Path.GetFileName(Source);
                            await Task.Run(() => File.Copy(Source, Path.Combine(Theme, Preview), true));
                        }
                        else
                        {
                            await Task.Run(async () => await ExtractResources(Preview, Theme));
                        }

                        SSTHI Info = new()
                        {
                            Tags = Tags,
                            License = null,
                            Arguments = null,
                            Preview = Preview,
                            Thumbnail = Thumbnail,
                            Type = SSDEWT.YouTube,
                            AppVersion = SSCHV.Get(),
                            Source = YouTubeUrl.Text,
                            Version = new(1, 0, 0, 0),
                            Title = YouTubeTitle.Text,
                            Author = YouTubeAuthor.Text,
                            Contact = YouTubeContact.Text,
                            Description = YouTubeDescription.Text
                        };

                        SSTHI.WriteJson(Path.Combine(Theme, SMR.SucroseInfo), Info);
                    }
                }
                else if (ApplicationCard.Visibility == Visibility.Visible)
                {
                    if (Path.GetExtension($"{ApplicationSource.Content}") != ".exe")
                    {
                        ApplicationSource.BorderBrush = Brushes.Crimson;
                        ApplicationSource.Focus();
                        return;
                    }
                    else if (string.IsNullOrEmpty(ApplicationTitle.Text))
                    {
                        ApplicationTitle.Focus();
                        return;
                    }
                    else if (string.IsNullOrEmpty(ApplicationDescription.Text))
                    {
                        ApplicationDescription.Focus();
                        return;
                    }
                    else if (string.IsNullOrEmpty(ApplicationAuthor.Text))
                    {
                        ApplicationAuthor.Focus();
                        return;
                    }
                    else if (!SSTHV.IsUrl(ApplicationContact.Text) && !SSTHV.IsMail(ApplicationContact.Text))
                    {
                        ApplicationContact.Focus();
                        return;
                    }
                    else
                    {
                        string Theme;
                        string[] Tags;
                        string Arguments;
                        string Preview = "Preview.gif";
                        string Thumbnail = "Thumbnail.jpg";

                        if (string.IsNullOrEmpty(ApplicationTags.Text))
                        {
                            Tags = Array.Empty<string>();
                        }
                        else
                        {
                            if (ApplicationTags.Text.Contains(','))
                            {
                                Tags = ApplicationTags.Text.Split(',').Select(Tag => Tag.TrimStart().TrimEnd()).ToArray();

                                if (Tags.Count() is < 1 or > 5)
                                {
                                    ApplicationTags.Focus();
                                    return;
                                }
                                else if (Tags.Any(Tag => Tag.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(Tag)))
                                {
                                    ApplicationTags.Focus();
                                    return;
                                }
                            }
                            else
                            {
                                if (ApplicationTags.Text.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(ApplicationTags.Text))
                                {
                                    ApplicationTags.Focus();
                                    return;
                                }
                                else
                                {
                                    Tags = new[]
                                    {
                                        ApplicationTags.Text.TrimStart().TrimEnd()
                                    };
                                }
                            }
                        }

                        if (string.IsNullOrEmpty(ApplicationArguments.Text))
                        {
                            Arguments = null;
                        }
                        else
                        {
                            if (ApplicationArguments.Text.Length is > 250 || string.IsNullOrWhiteSpace(ApplicationArguments.Text))
                            {
                                ApplicationArguments.Focus();
                                return;
                            }
                            else
                            {
                                Arguments = ApplicationArguments.Text;
                            }
                        }

                        do
                        {
                            SPMI.LibraryService.Theme = SHG.GenerateString(SMMM.Chars, 25, SMR.Randomise);
                            Theme = Path.Combine(SMML.LibraryLocation, SPMI.LibraryService.Theme);
                        } while (Directory.Exists(Theme));

                        Directory.CreateDirectory(Theme);

                        SSSHC.Folder(Path.GetDirectoryName($"{ApplicationSource.Content}"), Theme, false);

                        if (File.Exists($"{ApplicationThumbnail.Content}"))
                        {
                            string Source = $"{ApplicationThumbnail.Content}";
                            Thumbnail = "t_" + Path.GetFileName(Source);
                            await Task.Run(() => File.Copy(Source, Path.Combine(Theme, Thumbnail), true));
                        }
                        else
                        {
                            await Task.Run(async () => await ExtractResources(Thumbnail, Theme));
                        }

                        if (File.Exists($"{ApplicationPreview.Content}"))
                        {
                            string Source = $"{ApplicationPreview.Content}";
                            Preview = "p_" + Path.GetFileName(Source);
                            await Task.Run(() => File.Copy(Source, Path.Combine(Theme, Preview), true));
                        }
                        else
                        {
                            await Task.Run(async () => await ExtractResources(Preview, Theme));
                        }

                        SSTHI Info = new()
                        {
                            Tags = Tags,
                            License = null,
                            Preview = Preview,
                            Arguments = Arguments,
                            Thumbnail = Thumbnail,
                            AppVersion = SSCHV.Get(),
                            Type = SSDEWT.Application,
                            Version = new(1, 0, 0, 0),
                            Title = ApplicationTitle.Text,
                            Author = ApplicationAuthor.Text,
                            Contact = ApplicationContact.Text,
                            Description = ApplicationDescription.Text,
                            Source = Path.GetFileName($"{ApplicationSource.Content}")
                        };

                        SSTHI.WriteJson(Path.Combine(Theme, SMR.SucroseInfo), Info);
                    }
                }
            }

            base.OnButtonClick(Button);
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}