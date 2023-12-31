﻿using System.IO;
using System.Reflection;
using System.Windows;
using System.Windows.Input;
using System.Windows.Media;
using Wpf.Ui.Controls;
using OpenFileDialog = Microsoft.Win32.OpenFileDialog;
using SEAT = Skylark.Enum.AssemblyType;
using SHA = Skylark.Helper.Assemblies;
using SHG = Skylark.Helper.Generator;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SPETL = Sucrose.Portal.Extension.ThumbnailLoader;
using SPMI = Sucrose.Portal.Manage.Internal;
using SRER = Sucrose.Resources.Extension.Resources;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
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
                        string Extension = Path.GetExtension(Record).ToLowerInvariant();

                        if (Extension is ".gif")
                        {
                            GifImagine.Source = await Loader.LoadAsync(Record);
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

        private void ThemePreview_Click(object sender, RoutedEventArgs e)
        {
            Button Button = sender as Button;

            string Startup = File.Exists($"{Button.Content}") ? Path.GetDirectoryName($"{Button.Content}") : SMR.DesktopPath;

            OpenFileDialog FileDialog = new()
            {
                Filter = SRER.GetValue("Portal", "ThemeCreate", "ThemePreview", "Filter"),
                FilterIndex = 1,

                Title = SRER.GetValue("Portal", "ThemeCreate", "ThemePreview", "Title"),

                InitialDirectory = Startup
            };

            if (FileDialog.ShowDialog() == true)
            {
                Button.Content = FileDialog.FileName;
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
                        string Extension = Path.GetExtension(Record).ToLowerInvariant();

                        if (Extension is ".mp4" or ".avi" or ".mov" or ".mkv" or ".ogv" or ".flv" or ".wmv" or ".hevc" or ".webm" or ".mpeg" or ".mpeg1" or ".mpeg2" or ".mpeg4")
                        {
                            VideoImagine.Source = await Loader.LoadAsync(Record);
                            VideoDelete.Visibility = Visibility.Visible;
                            VideoIcon.Visibility = Visibility.Collapsed;
                            VideoText.Visibility = Visibility.Collapsed;
                            VideoRectangle.Stroke = Brushes.SeaGreen;
                            break;
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

        private void ThemeThumbnail_Click(object sender, RoutedEventArgs e)
        {
            Button Button = sender as Button;

            string Startup = File.Exists($"{Button.Content}") ? Path.GetDirectoryName($"{Button.Content}") : SMR.DesktopPath;

            OpenFileDialog FileDialog = new()
            {
                Filter = SRER.GetValue("Portal", "ThemeCreate", "ThemeThumbnail", "Filter"),
                FilterIndex = 1,

                Title = SRER.GetValue("Portal", "ThemeCreate", "ThemeThumbnail", "Title"),

                InitialDirectory = Startup
            };

            if (FileDialog.ShowDialog() == true)
            {
                Button.Content = FileDialog.FileName;
            }
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
                            Tags = Tags,
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
                            Theme = Path.Combine(SMMM.LibraryLocation, SPMI.LibraryService.Theme);
                        } while (File.Exists(Theme));

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
                            Theme = Path.Combine(SMMM.LibraryLocation, SPMI.LibraryService.Theme);
                        } while (File.Exists(Theme));

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
                            await Task.Run(async () => await ExtractResources(Thumbnail, Theme));
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
                            Theme = Path.Combine(SMMM.LibraryLocation, SPMI.LibraryService.Theme);
                        } while (File.Exists(Theme));

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