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
using SMMRC = Sucrose.Memory.Manage.Readonly.Content;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMMVA = Sucrose.Memory.Manage.Valuable.App;
using SPETC = Sucrose.Portal.Extension.ThemeCreate;
using SPETL = Sucrose.Portal.Extension.ThumbnailLoader;
using SPMI = Sucrose.Portal.Manage.Internal;
using SRER = Sucrose.Resources.Extension.Resources;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSSHA = Sucrose.Shared.Space.Helper.Access;
using SSSHC = Sucrose.Shared.Space.Helper.Copy;
using SSSHF = Sucrose.Shared.Space.Helper.Filing;
using SSSHL = Sucrose.Shared.Space.Helper.Lock;
using SSTHF = Sucrose.Shared.Theme.Helper.Filter;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;
using SSWEW = Sucrose.Shared.Watchdog.Extension.Watch;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeCreate.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeCreate : ContentDialog, IDisposable
    {
        private readonly SPETL Loader = new();

        public string InitialFilePath { get; set; }

        public string InitialUrl { get; set; }

        public ThemeCreate(ContentDialogHost? contentDialogHost) : base(contentDialogHost)
        {
            InitializeComponent();
        }

        private async Task CreateTheme()
        {
            Root.IsEnabled = false;
            Control FocusingControl = null;
            IsPrimaryButtonEnabled = false;
            IsSecondaryButtonEnabled = true;
            CloseButtonAppearance = ControlAppearance.Dark;

            await Application.Current.Dispatcher.Invoke(async () =>
            {
                if (!Directory.Exists(SMML.Location))
                {
                    Directory.CreateDirectory(SMML.Location);
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
                        FocusingControl = GifTitle;
                        return;
                    }
                    else if (string.IsNullOrEmpty(GifDescription.Text))
                    {
                        FocusingControl = GifDescription;
                        return;
                    }
                    else if (string.IsNullOrEmpty(GifAuthor.Text))
                    {
                        FocusingControl = GifAuthor;
                        return;
                    }
                    else if (!SSTHV.IsUrl(GifContact.Text) && !SSTHV.IsMail(GifContact.Text))
                    {
                        FocusingControl = GifContact;
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
                                    FocusingControl = GifTags;
                                    return;
                                }
                                else if (Tags.Any(Tag => Tag.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(Tag)))
                                {
                                    FocusingControl = GifTags;
                                    return;
                                }
                            }
                            else
                            {
                                if (GifTags.Text.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(GifTags.Text))
                                {
                                    FocusingControl = GifTags;
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
                            SPMI.LibraryService.Theme = SHG.GenerateString(SMMVA.Chars, 25, SMMRG.Randomise);
                            Theme = Path.Combine(SMML.Location, SPMI.LibraryService.Theme);
                        } while (Directory.Exists(Theme));

                        Directory.CreateDirectory(Theme);

                        if (File.Exists(Gif.LocalPath))
                        {
                            Name = "s_" + Path.GetFileName(Gif.LocalPath);
                            await Task.Run(() => SSSHF.CopyBuffer(Gif.LocalPath, Path.Combine(Theme, Name)));
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
                            await Task.Run(() => SSSHF.CopyBuffer(Source, Path.Combine(Theme, Thumbnail)));
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
                            await Task.Run(() => SSSHF.CopyBuffer(Source, Path.Combine(Theme, Preview)));
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
                            Localization = null,
                            Thumbnail = Thumbnail,
                            Title = GifTitle.Text,
                            Author = GifAuthor.Text,
                            AppVersion = SSCHV.Get(),
                            Version = new(1, 0, 0, 0),
                            Contact = GifContact.Text,
                            Description = GifDescription.Text
                        };

                        SSTHI.Write(Path.Combine(Theme, SMMRC.SucroseInfo), Info);

                        Hide(ContentDialogResult.Primary);
                    }
                }
                else if (UrlCard.Visibility == Visibility.Visible)
                {
                    if (!SSTHV.IsUrl(UrlUrl.Text))
                    {
                        FocusingControl = UrlUrl;
                        return;
                    }
                    else if (string.IsNullOrEmpty(UrlTitle.Text))
                    {
                        FocusingControl = UrlTitle;
                        return;
                    }
                    else if (string.IsNullOrEmpty(UrlDescription.Text))
                    {
                        FocusingControl = UrlDescription;
                        return;
                    }
                    else if (string.IsNullOrEmpty(UrlAuthor.Text))
                    {
                        FocusingControl = UrlAuthor;
                        return;
                    }
                    else if (!SSTHV.IsUrl(UrlContact.Text) && !SSTHV.IsMail(UrlContact.Text))
                    {
                        FocusingControl = UrlContact;
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
                                    FocusingControl = UrlTags;
                                    return;
                                }
                                else if (Tags.Any(Tag => Tag.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(Tag)))
                                {
                                    FocusingControl = UrlTags;
                                    return;
                                }
                            }
                            else
                            {
                                if (UrlTags.Text.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(UrlTags.Text))
                                {
                                    FocusingControl = UrlTags;
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
                            SPMI.LibraryService.Theme = SHG.GenerateString(SMMVA.Chars, 25, SMMRG.Randomise);
                            Theme = Path.Combine(SMML.Location, SPMI.LibraryService.Theme);
                        } while (Directory.Exists(Theme));

                        Directory.CreateDirectory(Theme);

                        if (File.Exists($"{UrlThumbnail.Content}"))
                        {
                            string Source = $"{UrlThumbnail.Content}";
                            Thumbnail = "t_" + Path.GetFileName(Source);
                            await Task.Run(() => SSSHF.CopyBuffer(Source, Path.Combine(Theme, Thumbnail)));
                        }
                        else
                        {
                            await Task.Run(async () => await ExtractResources(Thumbnail, Theme));
                        }

                        if (File.Exists($"{UrlPreview.Content}"))
                        {
                            string Source = $"{UrlPreview.Content}";
                            Preview = "p_" + Path.GetFileName(Source);
                            await Task.Run(() => SSSHF.CopyBuffer(Source, Path.Combine(Theme, Preview)));
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
                            Localization = null,
                            Source = UrlUrl.Text,
                            Thumbnail = Thumbnail,
                            Title = UrlTitle.Text,
                            Author = UrlAuthor.Text,
                            AppVersion = SSCHV.Get(),
                            Version = new(1, 0, 0, 0),
                            Contact = UrlContact.Text,
                            Description = UrlDescription.Text
                        };

                        SSTHI.Write(Path.Combine(Theme, SMMRC.SucroseInfo), Info);

                        Hide(ContentDialogResult.Primary);
                    }
                }
                else if (WebCard.Visibility == Visibility.Visible)
                {
                    if (!SSTHF.WebExtension($"{WebSource.Content}"))
                    {
                        WebSource.BorderBrush = Brushes.Crimson;
                        FocusingControl = WebSource;
                        return;
                    }
                    else if (string.IsNullOrEmpty(WebTitle.Text))
                    {
                        FocusingControl = WebTitle;
                        return;
                    }
                    else if (string.IsNullOrEmpty(WebDescription.Text))
                    {
                        FocusingControl = WebDescription;
                        return;
                    }
                    else if (string.IsNullOrEmpty(WebAuthor.Text))
                    {
                        FocusingControl = WebAuthor;
                        return;
                    }
                    else if (!SSTHV.IsUrl(WebContact.Text) && !SSTHV.IsMail(WebContact.Text))
                    {
                        FocusingControl = WebContact;
                        return;
                    }
                    else if (!SSSHL.Directory(Path.GetDirectoryName($"{WebSource.Content}")))
                    {
                        MessageBox Warning = new()
                        {
                            Title = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Title"),
                            Content = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Message"),
                            CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Close")
                        };

                        await Warning.ShowDialogAsync();
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
                                    FocusingControl = WebTags;
                                    return;
                                }
                                else if (Tags.Any(Tag => Tag.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(Tag)))
                                {
                                    FocusingControl = WebTags;
                                    return;
                                }
                            }
                            else
                            {
                                if (WebTags.Text.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(WebTags.Text))
                                {
                                    FocusingControl = WebTags;
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
                            SPMI.LibraryService.Theme = SHG.GenerateString(SMMVA.Chars, 25, SMMRG.Randomise);
                            Theme = Path.Combine(SMML.Location, SPMI.LibraryService.Theme);
                        } while (Directory.Exists(Theme));

                        Directory.CreateDirectory(Theme);

                        SSSHC.Folder(Path.GetDirectoryName($"{WebSource.Content}"), Theme, false);

                        if (File.Exists($"{WebThumbnail.Content}"))
                        {
                            string Source = $"{WebThumbnail.Content}";
                            Thumbnail = "t_" + Path.GetFileName(Source);
                            await Task.Run(() => SSSHF.CopyBuffer(Source, Path.Combine(Theme, Thumbnail)));
                        }
                        else
                        {
                            await Task.Run(async () => await ExtractResources(Thumbnail, Theme));
                        }

                        if (File.Exists($"{WebPreview.Content}"))
                        {
                            string Source = $"{WebPreview.Content}";
                            Preview = "p_" + Path.GetFileName(Source);
                            await Task.Run(() => SSSHF.CopyBuffer(Source, Path.Combine(Theme, Preview)));
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
                            Localization = null,
                            Thumbnail = Thumbnail,
                            Title = WebTitle.Text,
                            Author = WebAuthor.Text,
                            AppVersion = SSCHV.Get(),
                            Version = new(1, 0, 0, 0),
                            Contact = WebContact.Text,
                            Description = WebDescription.Text,
                            Source = Path.GetFileName($"{WebSource.Content}")
                        };

                        SSTHI.Write(Path.Combine(Theme, SMMRC.SucroseInfo), Info);

                        Hide(ContentDialogResult.Primary);
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
                        FocusingControl = VideoTitle;
                        return;
                    }
                    else if (string.IsNullOrEmpty(VideoDescription.Text))
                    {
                        FocusingControl = VideoDescription;
                        return;
                    }
                    else if (string.IsNullOrEmpty(VideoAuthor.Text))
                    {
                        FocusingControl = VideoAuthor;
                        return;
                    }
                    else if (!SSTHV.IsUrl(VideoContact.Text) && !SSTHV.IsMail(VideoContact.Text))
                    {
                        FocusingControl = VideoContact;
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
                                    FocusingControl = VideoTags;
                                    return;
                                }
                                else if (Tags.Any(Tag => Tag.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(Tag)))
                                {
                                    FocusingControl = VideoTags;
                                    return;
                                }
                            }
                            else
                            {
                                if (VideoTags.Text.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(VideoTags.Text))
                                {
                                    FocusingControl = VideoTags;
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
                            SPMI.LibraryService.Theme = SHG.GenerateString(SMMVA.Chars, 25, SMMRG.Randomise);
                            Theme = Path.Combine(SMML.Location, SPMI.LibraryService.Theme);
                        } while (Directory.Exists(Theme));

                        Directory.CreateDirectory(Theme);

                        if (File.Exists(Video.LocalPath))
                        {
                            Name = "s_" + Path.GetFileName(Video.LocalPath);
                            await Task.Run(() => SSSHF.CopyBuffer(Video.LocalPath, Path.Combine(Theme, Name)));
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
                            await Task.Run(() => SSSHF.CopyBuffer(Source, Path.Combine(Theme, Thumbnail)));
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
                            await Task.Run(() => SSSHF.CopyBuffer(Source, Path.Combine(Theme, Preview)));
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
                            Localization = null,
                            Type = SSDEWT.Video,
                            Thumbnail = Thumbnail,
                            Title = VideoTitle.Text,
                            AppVersion = SSCHV.Get(),
                            Version = new(1, 0, 0, 0),
                            Author = VideoAuthor.Text,
                            Contact = VideoContact.Text,
                            Description = VideoDescription.Text
                        };

                        SSTHI.Write(Path.Combine(Theme, SMMRC.SucroseInfo), Info);

                        Hide(ContentDialogResult.Primary);
                    }
                }
                else if (YouTubeCard.Visibility == Visibility.Visible)
                {
                    if (!SSTHV.IsYouTubeAll(YouTubeUrl.Text))
                    {
                        FocusingControl = YouTubeUrl;
                        return;
                    }
                    else if (string.IsNullOrEmpty(YouTubeTitle.Text))
                    {
                        FocusingControl = YouTubeTitle;
                        return;
                    }
                    else if (string.IsNullOrEmpty(YouTubeDescription.Text))
                    {
                        FocusingControl = YouTubeDescription;
                        return;
                    }
                    else if (string.IsNullOrEmpty(YouTubeAuthor.Text))
                    {
                        FocusingControl = YouTubeAuthor;
                        return;
                    }
                    else if (!SSTHV.IsUrl(YouTubeContact.Text) && !SSTHV.IsMail(YouTubeContact.Text))
                    {
                        FocusingControl = YouTubeContact;
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
                                    FocusingControl = YouTubeTags;
                                    return;
                                }
                                else if (Tags.Any(Tag => Tag.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(Tag)))
                                {
                                    FocusingControl = YouTubeTags;
                                    return;
                                }
                            }
                            else
                            {
                                if (YouTubeTags.Text.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(YouTubeTags.Text))
                                {
                                    FocusingControl = YouTubeTags;
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
                            SPMI.LibraryService.Theme = SHG.GenerateString(SMMVA.Chars, 25, SMMRG.Randomise);
                            Theme = Path.Combine(SMML.Location, SPMI.LibraryService.Theme);
                        } while (Directory.Exists(Theme));

                        Directory.CreateDirectory(Theme);

                        if (File.Exists($"{YouTubeThumbnail.Content}"))
                        {
                            string Source = $"{YouTubeThumbnail.Content}";
                            Thumbnail = "t_" + Path.GetFileName(Source);
                            await Task.Run(() => SSSHF.CopyBuffer(Source, Path.Combine(Theme, Thumbnail)));
                        }
                        else
                        {
                            await Task.Run(async () => await ExtractResources(Thumbnail, Theme));
                        }

                        if (File.Exists($"{YouTubePreview.Content}"))
                        {
                            string Source = $"{YouTubePreview.Content}";
                            Preview = "p_" + Path.GetFileName(Source);
                            await Task.Run(() => SSSHF.CopyBuffer(Source, Path.Combine(Theme, Preview)));
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
                            Localization = null,
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

                        SSTHI.Write(Path.Combine(Theme, SMMRC.SucroseInfo), Info);

                        Hide(ContentDialogResult.Primary);
                    }
                }
                else if (ApplicationCard.Visibility == Visibility.Visible)
                {
                    if (!SSTHF.AppExtension($"{ApplicationSource.Content}"))
                    {
                        ApplicationSource.BorderBrush = Brushes.Crimson;
                        FocusingControl = ApplicationSource;
                        return;
                    }
                    else if (string.IsNullOrEmpty(ApplicationTitle.Text))
                    {
                        FocusingControl = ApplicationTitle;
                        return;
                    }
                    else if (string.IsNullOrEmpty(ApplicationDescription.Text))
                    {
                        FocusingControl = ApplicationDescription;
                        return;
                    }
                    else if (string.IsNullOrEmpty(ApplicationAuthor.Text))
                    {
                        FocusingControl = ApplicationAuthor;
                        return;
                    }
                    else if (!SSTHV.IsUrl(ApplicationContact.Text) && !SSTHV.IsMail(ApplicationContact.Text))
                    {
                        FocusingControl = ApplicationContact;
                        return;
                    }
                    else if (!SSSHL.Directory(Path.GetDirectoryName($"{ApplicationSource.Content}")))
                    {
                        MessageBox Warning = new()
                        {
                            Title = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Title"),
                            Content = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Message"),
                            CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Close")
                        };

                        await Warning.ShowDialogAsync();
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
                                    FocusingControl = ApplicationTags;
                                    return;
                                }
                                else if (Tags.Any(Tag => Tag.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(Tag)))
                                {
                                    FocusingControl = ApplicationTags;
                                    return;
                                }
                            }
                            else
                            {
                                if (ApplicationTags.Text.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(ApplicationTags.Text))
                                {
                                    FocusingControl = ApplicationTags;
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
                                FocusingControl = ApplicationArguments;
                                return;
                            }
                            else
                            {
                                Arguments = ApplicationArguments.Text;
                            }
                        }

                        do
                        {
                            SPMI.LibraryService.Theme = SHG.GenerateString(SMMVA.Chars, 25, SMMRG.Randomise);
                            Theme = Path.Combine(SMML.Location, SPMI.LibraryService.Theme);
                        } while (Directory.Exists(Theme));

                        Directory.CreateDirectory(Theme);

                        SSSHC.Folder(Path.GetDirectoryName($"{ApplicationSource.Content}"), Theme, false);

                        if (File.Exists($"{ApplicationThumbnail.Content}"))
                        {
                            string Source = $"{ApplicationThumbnail.Content}";
                            Thumbnail = "t_" + Path.GetFileName(Source);
                            await Task.Run(() => SSSHF.CopyBuffer(Source, Path.Combine(Theme, Thumbnail)));
                        }
                        else
                        {
                            await Task.Run(async () => await ExtractResources(Thumbnail, Theme));
                        }

                        if (File.Exists($"{ApplicationPreview.Content}"))
                        {
                            string Source = $"{ApplicationPreview.Content}";
                            Preview = "p_" + Path.GetFileName(Source);
                            await Task.Run(() => SSSHF.CopyBuffer(Source, Path.Combine(Theme, Preview)));
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
                            Localization = null,
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

                        SSTHI.Write(Path.Combine(Theme, SMMRC.SucroseInfo), Info);

                        Hide(ContentDialogResult.Primary);
                    }
                }
            });

            Root.IsEnabled = true;
            FocusingControl?.Focus();
            IsPrimaryButtonEnabled = true;
            IsSecondaryButtonEnabled = false;
            CloseButtonAppearance = ControlAppearance.Secondary;
        }

        private void ProcessInitialUrl()
        {
            if (SSTHV.IsYouTubeAll(InitialUrl))
            {
                YouTubeCreate_Click(null, null);

                YouTubeUrl.Text = InitialUrl;
            }
            else if (SSTHV.IsUrl(InitialUrl))
            {
                UrlCreate_Click(null, null);

                UrlUrl.Text = InitialUrl;
            }
        }

        private async Task ProcessInitialFileAsync()
        {
            if (!File.Exists(InitialFilePath))
            {
                return;
            }

            if (!SSSHA.File(InitialFilePath))
            {
                return;
            }

            if (!SSSHL.File(InitialFilePath))
            {
                return;
            }

            if (SSTHF.GifExtension(InitialFilePath))
            {
                GifCreate_Click(null, null);

                GifDescription.Text = SPETC.GetDescription(Path.GetFileNameWithoutExtension(InitialFilePath), SSDEWT.Gif);
                GifTitle.Text = SPETC.GetTitle(Path.GetFileNameWithoutExtension(InitialFilePath));

                try
                {
                    GifImagine.Source = await Loader.LoadAsync(InitialFilePath);
                }
                catch (Exception Exception)
                {
                    await SSWEW.Watch_CatchException(Exception);
                }

                GifDelete.Visibility = Visibility.Visible;
                GifIcon.Visibility = Visibility.Collapsed;
                GifText.Visibility = Visibility.Collapsed;
                GifRectangle.Stroke = Brushes.SeaGreen;
            }
            else if (SSTHF.VideoExtension(InitialFilePath))
            {
                VideoCreate_Click(null, null);

                VideoDescription.Text = SPETC.GetDescription(Path.GetFileNameWithoutExtension(InitialFilePath), SSDEWT.Video);
                VideoTitle.Text = SPETC.GetTitle(Path.GetFileNameWithoutExtension(InitialFilePath));

                try
                {
                    VideoImagine.Source = await Loader.LoadAsync(InitialFilePath);
                }
                catch (Exception Exception)
                {
                    await SSWEW.Watch_CatchException(Exception);
                }

                VideoDelete.Visibility = Visibility.Visible;
                VideoIcon.Visibility = Visibility.Collapsed;
                VideoText.Visibility = Visibility.Collapsed;
                VideoRectangle.Stroke = Brushes.SeaGreen;
            }
            else if (SSTHF.WebExtension(InitialFilePath))
            {
                WebCreate_Click(null, null);

                WebSource.Content = InitialFilePath;
                WebSource.BorderBrush = WebThumbnail.BorderBrush;
                WebTitle.Text = SPETC.GetTitle(Path.GetFileNameWithoutExtension(InitialFilePath));
                WebDescription.Text = SPETC.GetDescription(Path.GetFileNameWithoutExtension(InitialFilePath), SSDEWT.Web);
            }
            else if (SSTHF.AppExtension(InitialFilePath))
            {
                ApplicationCreate_Click(null, null);

                ApplicationSource.Content = InitialFilePath;
                ApplicationSource.BorderBrush = ApplicationThumbnail.BorderBrush;
                ApplicationTitle.Text = SPETC.GetTitle(Path.GetFileNameWithoutExtension(InitialFilePath));
                ApplicationDescription.Text = SPETC.GetDescription(Path.GetFileNameWithoutExtension(InitialFilePath), SSDEWT.Application);
            }
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
            WebSourceClear_Click(null, null);
            ApplicationSourceClear_Click(null, null);

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
                            if (SSSHL.File(Record))
                            {
                                string Extension = Path.GetExtension(Record).ToLowerInvariant();

                                if (Extension is ".gif")
                                {
                                    GifDescription.Text = SPETC.GetDescription(Path.GetFileNameWithoutExtension(Record), SSDEWT.Gif);
                                    GifTitle.Text = SPETC.GetTitle(Path.GetFileNameWithoutExtension(Record));

                                    try
                                    {
                                        GifImagine.Source = await Loader.LoadAsync(Record);
                                    }
                                    catch (Exception Exception)
                                    {
                                        await SSWEW.Watch_CatchException(Exception);
                                    }

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
                                    Title = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Title"),
                                    Content = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Message"),
                                    CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Close")
                                };

                                await Warning.ShowDialogAsync();
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
                            if (SSSHL.File(Record))
                            {
                                string Extension = Path.GetExtension(Record).ToLowerInvariant();

                                if (Extension is ".mp4" or ".avi" or ".mov" or ".mkv" or ".ogv" or ".flv" or ".wmv" or ".hevc" or ".webm" or ".mpeg" or ".mpeg1" or ".mpeg2" or ".mpeg4")
                                {
                                    VideoDescription.Text = SPETC.GetDescription(Path.GetFileNameWithoutExtension(Record), SSDEWT.Video);
                                    VideoTitle.Text = SPETC.GetTitle(Path.GetFileNameWithoutExtension(Record));

                                    try
                                    {
                                        VideoImagine.Source = await Loader.LoadAsync(Record);
                                    }
                                    catch (Exception Exception)
                                    {
                                        await SSWEW.Watch_CatchException(Exception);
                                    }

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
                                    Title = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Title"),
                                    Content = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Message"),
                                    CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Close")
                                };

                                await Warning.ShowDialogAsync();
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

        private async void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            if (!string.IsNullOrEmpty(InitialFilePath))
            {
                await ProcessInitialFileAsync();
            }
            else if (!string.IsNullOrEmpty(InitialUrl))
            {
                ProcessInitialUrl();
            }
        }

        private void WebSourceClear_Click(object sender, RoutedEventArgs e)
        {
            WebSource.BorderBrush = WebThumbnail.BorderBrush;
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
                    if (SSSHL.File(FileDialog.FileName))
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
                            Title = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Title"),
                            Content = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Message"),
                            CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Close")
                        };

                        await Warning.ShowDialogAsync();
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
                    if (SSSHL.File(FileDialog.FileName))
                    {
                        Button.Content = FileDialog.FileName;
                    }
                    else
                    {
                        MessageBox Warning = new()
                        {
                            Title = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Title"),
                            Content = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Message"),
                            CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Close")
                        };

                        await Warning.ShowDialogAsync();
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
                    if (SSSHL.File(FileDialog.FileName))
                    {
                        Button.Content = FileDialog.FileName;
                    }
                    else
                    {
                        MessageBox Warning = new()
                        {
                            Title = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Title"),
                            Content = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Message"),
                            CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Close")
                        };

                        await Warning.ShowDialogAsync();
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

        private void ApplicationSourceClear_Click(object sender, RoutedEventArgs e)
        {
            ApplicationSource.BorderBrush = ApplicationThumbnail.BorderBrush;
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
                    if (SSSHL.File(FileDialog.FileName))
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
                            Title = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Title"),
                            Content = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Message"),
                            CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Close")
                        };

                        await Warning.ShowDialogAsync();
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
                    if (SSSHL.File(FileDialog.FileName))
                    {
                        GifDescription.Text = SPETC.GetDescription(Path.GetFileNameWithoutExtension(FileDialog.FileName), SSDEWT.Gif);
                        GifTitle.Text = SPETC.GetTitle(Path.GetFileNameWithoutExtension(FileDialog.FileName));

                        try
                        {
                            GifImagine.Source = await Loader.LoadAsync(FileDialog.FileName);
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }

                        GifDelete.Visibility = Visibility.Visible;
                        GifIcon.Visibility = Visibility.Collapsed;
                        GifText.Visibility = Visibility.Collapsed;
                        GifRectangle.Stroke = Brushes.SeaGreen;
                    }
                    else
                    {
                        MessageBox Warning = new()
                        {
                            Title = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Title"),
                            Content = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Message"),
                            CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Close")
                        };

                        await Warning.ShowDialogAsync();
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
                    if (SSSHL.File(FileDialog.FileName))
                    {
                        VideoDescription.Text = SPETC.GetDescription(Path.GetFileNameWithoutExtension(FileDialog.FileName), SSDEWT.Video);
                        VideoTitle.Text = SPETC.GetTitle(Path.GetFileNameWithoutExtension(FileDialog.FileName));

                        try
                        {
                            VideoImagine.Source = await Loader.LoadAsync(FileDialog.FileName);
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }

                        VideoDelete.Visibility = Visibility.Visible;
                        VideoIcon.Visibility = Visibility.Collapsed;
                        VideoText.Visibility = Visibility.Collapsed;
                        VideoRectangle.Stroke = Brushes.SeaGreen;
                    }
                    else
                    {
                        MessageBox Warning = new()
                        {
                            Title = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Title"),
                            Content = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Message"),
                            CloseButtonText = SRER.GetValue("Portal", "ThemeCreate", "Lock", "Close")
                        };

                        await Warning.ShowDialogAsync();
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

        protected override async void OnButtonClick(ContentDialogButton Button)
        {
            if (Button == ContentDialogButton.Primary)
            {
                await CreateTheme();

                return;
            }
            else if (IsSecondaryButtonEnabled || Button == ContentDialogButton.Secondary)
            {
                return;
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