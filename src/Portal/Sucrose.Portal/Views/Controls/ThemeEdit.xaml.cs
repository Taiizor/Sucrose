using System.IO;
using System.Windows;
using Wpf.Ui.Controls;
using SECNT = Skylark.Enum.ClearNumericType;
using SEMST = Skylark.Enum.ModeStorageType;
using SEST = Skylark.Enum.StorageType;
using SMR = Sucrose.Memory.Readonly;
using SHC = Skylark.Helper.Culture;
using SHN = Skylark.Helper.Numeric;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPEIL = Sucrose.Portal.Extension.ImageLoader;
using SSESSE = Skylark.Standard.Extension.Storage.StorageExtension;
using SSSHS = Sucrose.Shared.Space.Helper.Size;
using SSSSS = Skylark.Struct.Storage.StorageStruct;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;
using System.Windows.Input;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;

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
            string ImagePath = Path.Combine(Theme, Info.Thumbnail);

            if (File.Exists(ImagePath))
            {
                ThemeThumbnail.Source = Loader.LoadOptimal(ImagePath);
            }

            ThemeTitle.Text = Info.Title;
            ThemeAuthor.Text = Info.Author;
            ThemeContact.Text = Info.Contact;
            ThemeArguments.Text = Info.Arguments;
            ThemeDescription.Text = Info.Description;

            if (Info.Type != SSDEWT.Application)
            {
                Arguments.Visibility = Visibility.Collapsed;
            }
        }

        private void ContentDialog_PreviewKeyDown(object sender, KeyEventArgs e)
        {
            if (e.Key == Key.Escape && (ThemeTitle.IsFocused || ThemeAuthor.IsFocused || ThemeContact.IsFocused || ThemeArguments.IsFocused || ThemeDescription.IsFocused))
            {
                e.Handled = true;
            }
            else if (e.Key == Key.Enter && (ThemeTitle.IsFocused || ThemeAuthor.IsFocused || ThemeContact.IsFocused || ThemeArguments.IsFocused))
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
                else
                {
                    Info.Title = ThemeTitle.Text;
                    Info.Author = ThemeAuthor.Text;
                    Info.Contact = ThemeContact.Text;
                    Info.Description = ThemeDescription.Text;

                    if (string.IsNullOrEmpty(ThemeArguments.Text))
                    {
                        Info.Arguments = null;
                    }
                    else
                    {
                        Info.Arguments = ThemeArguments.Text;
                    }

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