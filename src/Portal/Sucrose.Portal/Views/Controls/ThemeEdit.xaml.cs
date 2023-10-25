using System.IO;
using System.Windows;
using System.Windows.Input;
using Wpf.Ui.Controls;
using SMR = Sucrose.Memory.Readonly;
using SPEIL = Sucrose.Portal.Extension.ImageLoader;
using SPMI = Sucrose.Portal.Manage.Internal;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSSHT = Sucrose.Shared.Space.Helper.Tags;
using SSSHV = Sucrose.Shared.Space.Helper.Versionly;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeEdit.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeEdit : ContentDialog, IDisposable
    {
        private readonly SPEIL Loader = new();
        internal string Theme = string.Empty;
        internal SSTHI Info = new();

        public ThemeEdit() : base(SPMI.ContentDialogService.GetContentPresenter())
        {
            InitializeComponent();
        }

        private void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            ThemeTitle.Text = Info.Title;
            ThemeAuthor.Text = Info.Author;
            ThemeContact.Text = Info.Contact;
            ThemeArguments.Text = Info.Arguments;
            ThemeDescription.Text = Info.Description;
            ThemeTags.Text = SSSHT.Join(Info.Tags, ",", string.Empty);

            if (Info.Type != SSDEWT.Application)
            {
                Arguments.Visibility = Visibility.Collapsed;
            }

            string ImagePath = Path.Combine(Theme, Info.Thumbnail);

            if (File.Exists(ImagePath))
            {
                ThemeThumbnail.Source = Loader.LoadOptimal(ImagePath, true, 600);
            }
        }

        private void ContentDialog_PreviewKeyDown(object sender, KeyEventArgs e)
        {
            if ((e.Key == Key.Enter || e.Key == Key.Escape) && (ThemeTitle.IsFocused || ThemeAuthor.IsFocused || ThemeContact.IsFocused || ThemeArguments.IsFocused || ThemeDescription.IsFocused))
            {
                e.Handled = true;
            }
        }

        protected override void OnButtonClick(ContentDialogButton Button)
        {
            if (Button == ContentDialogButton.Primary)
            {
                if (string.IsNullOrEmpty(ThemeTitle.Text))
                {
                    ThemeTitle.Focus();
                    return;
                }
                else if (string.IsNullOrEmpty(ThemeAuthor.Text))
                {
                    ThemeAuthor.Focus();
                    return;
                }
                else if (string.IsNullOrEmpty(ThemeDescription.Text))
                {
                    ThemeDescription.Focus();
                    return;
                }
                else if (!SSTHV.IsUrl(ThemeContact.Text) && !SSTHV.IsMail(ThemeContact.Text))
                {
                    ThemeContact.Focus();
                    return;
                }
                else
                {
                    if (string.IsNullOrEmpty(ThemeTags.Text))
                    {
                        Info.Tags = null;
                    }
                    else
                    {
                        if (ThemeTags.Text.Contains(','))
                        {
                            Info.Tags = ThemeTags.Text.Split(',').Select(Tag => Tag.TrimStart().TrimEnd()).ToArray();

                            if (Info.Tags.Count() is < 1 or > 5)
                            {
                                ThemeTags.Focus();
                                return;
                            }
                            else if (Info.Tags.Any(Tag => Tag.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(Tag)))
                            {
                                ThemeTags.Focus();
                                return;
                            }
                        }
                        else
                        {
                            if (ThemeTags.Text.Length is < 1 or > 20 || string.IsNullOrWhiteSpace(ThemeTags.Text))
                            {
                                ThemeTags.Focus();
                                return;
                            }
                            else
                            {
                                Info.Tags = new[]
                                {
                                    ThemeTags.Text.TrimStart().TrimEnd()
                                };
                            }
                        }
                    }

                    if (string.IsNullOrEmpty(ThemeArguments.Text))
                    {
                        Info.Arguments = null;
                    }
                    else
                    {
                        if (ThemeArguments.Text.Length is > 250 || string.IsNullOrWhiteSpace(ThemeArguments.Text))
                        {
                            ThemeTags.Focus();
                            return;
                        }
                        else
                        {
                            Info.Arguments = ThemeArguments.Text;
                        }
                    }


                    Info.Title = ThemeTitle.Text;
                    Info.Author = ThemeAuthor.Text;
                    Info.Contact = ThemeContact.Text;
                    Info.Description = ThemeDescription.Text;
                    Info.Version = SSSHV.Increment(Info.Version);

                    SSTHI.WriteJson(Path.Combine(Theme, SMR.SucroseInfo), Info);
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