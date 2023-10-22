using System.IO;
using System.Reflection;
using System.Windows;
using System.Windows.Input;
using System.Windows.Media;
using Wpf.Ui.Controls;
using XamlAnimatedGif;
using OpenFileDialog = Microsoft.Win32.OpenFileDialog;
using SEAT = Skylark.Enum.AssemblyType;
using SHA = Skylark.Helper.Assemblies;
using SHG = Skylark.Helper.Generator;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SPMI = Sucrose.Portal.Manage.Internal;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeCreate.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeCreate : ContentDialog, IDisposable
    {
        public ThemeCreate() : base(SPMI.ContentDialogService.GetContentPresenter())
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
        }

        private void GifArea_Drop(object sender, DragEventArgs e)
        {
            GifRectangle.Stroke = SSRER.GetResource<Brush>("TextFillColorDisabledBrush");

            if (e.Data.GetDataPresent(DataFormats.FileDrop))
            {
                string[] Files = (string[])e.Data.GetData(DataFormats.FileDrop);

                if (Files.Any())
                {
                    foreach (string Record in Files)
                    {
                        string Extension = Path.GetExtension(Record).ToLowerInvariant();

                        if (Extension is ".gif" or ".GIF")
                        {
                            AnimationBehavior.SetSourceUri(GifImagine, new(Record));
                            GifDelete.Visibility = Visibility.Visible;
                            GifIcon.Visibility = Visibility.Collapsed;
                            GifText.Visibility = Visibility.Collapsed;
                            GifRectangle.Stroke = Brushes.SeaGreen;
                            break;
                        }
                    }
                }
            }
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
            if (string.IsNullOrEmpty($"{AnimationBehavior.GetSourceUri(GifImagine)}"))
            {
                GifRectangle.Stroke = SSRER.GetResource<Brush>("TextFillColorDisabledBrush");
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
        }

        private void UrlCreate_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = true;
            UrlCard.Visibility = Visibility.Visible;
            CreateCard.Visibility = Visibility.Collapsed;
        }

        private void WebCreate_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = true;
            WebCard.Visibility = Visibility.Visible;
            CreateCard.Visibility = Visibility.Collapsed;
        }

        private void GifDelete_Click(object sender, RoutedEventArgs e)
        {
            GifIcon.Visibility = Visibility.Visible;
            GifText.Visibility = Visibility.Visible;
            GifDelete.Visibility = Visibility.Collapsed;
            AnimationBehavior.SetSourceUri(GifImagine, null);
            GifRectangle.Stroke = SSRER.GetResource<Brush>("TextFillColorDisabledBrush");
        }

        private void GifPreview_Click(object sender, RoutedEventArgs e)
        {
            string Startup = File.Exists($"{GifPreview.Content}") ? Path.GetDirectoryName($"{GifPreview.Content}") : SMR.DesktopPath;

            OpenFileDialog FileDialog = new()
            {
                Filter = SSRER.GetValue("Portal", "ThemeCreate", "ThemePreview", "Filter"),
                FilterIndex = 1,

                Title = SSRER.GetValue("Portal", "ThemeCreate", "ThemePreview", "Title"),

                InitialDirectory = Startup
            };

            if (FileDialog.ShowDialog() == true)
            {
                GifPreview.Content = FileDialog.FileName;
            }
        }

        private void VideoCreate_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = true;
            VideoCard.Visibility = Visibility.Visible;
            CreateCard.Visibility = Visibility.Collapsed;
        }

        private void GifThumbnail_Click(object sender, RoutedEventArgs e)
        {
            string Startup = File.Exists($"{GifThumbnail.Content}") ? Path.GetDirectoryName($"{GifThumbnail.Content}") : SMR.DesktopPath;

            OpenFileDialog FileDialog = new()
            {
                Filter = SSRER.GetValue("Portal", "ThemeCreate", "ThemeThumbnail", "Filter"),
                FilterIndex = 1,

                Title = SSRER.GetValue("Portal", "ThemeCreate", "ThemeThumbnail", "Title"),

                InitialDirectory = Startup
            };

            if (FileDialog.ShowDialog() == true)
            {
                GifThumbnail.Content = FileDialog.FileName;
            }
        }

        private void YouTubeCreate_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = true;
            YouTubeCard.Visibility = Visibility.Visible;
            CreateCard.Visibility = Visibility.Collapsed;
        }

        private void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            //
        }

        private void ApplicationCreate_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = true;
            CreateCard.Visibility = Visibility.Collapsed;
            ApplicationCard.Visibility = Visibility.Visible;
        }

        private void ContentDialog_PreviewKeyDown(object sender, KeyEventArgs e)
        {
            if (e.Key is Key.Enter or Key.Escape)
            {
                e.Handled = true;
            }
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

        protected override async void OnButtonClick(ContentDialogButton Button)
        {
            if (Button == ContentDialogButton.Primary)
            {
                if (!Directory.Exists(SMMM.LibraryLocation))
                {
                    Directory.CreateDirectory(SMMM.LibraryLocation);
                }

                if (GifCard.Visibility == Visibility.Visible)
                {
                    Uri Gif = AnimationBehavior.GetSourceUri(GifImagine);

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
                        string Preview = "Preview.gif";
                        string Thumbnail = "Thumbnail.jpg";

                        do
                        {
                            SPMI.LibraryService.Theme = SHG.GenerateString(SMMM.Chars, 25, SMR.Randomise);
                            Theme = Path.Combine(SMMM.LibraryLocation, SPMI.LibraryService.Theme);
                        } while (File.Exists(Theme));

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
                            await Task.Run(async () => await ExtractResources(Thumbnail, Theme));
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
                            Source = Name,
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