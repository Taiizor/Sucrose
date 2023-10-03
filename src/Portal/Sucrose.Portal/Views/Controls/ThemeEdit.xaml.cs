using System.IO;
using System.Windows;
using Wpf.Ui.Controls;
using SECNT = Skylark.Enum.ClearNumericType;
using SEMST = Skylark.Enum.ModeStorageType;
using SEST = Skylark.Enum.StorageType;
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
            ThemeDescription.GotFocus += (s, e) => ThemeTitle.Text = ThemeDescription.IsFocused.ToString();
            ThemeDescription.LostFocus += (s, e) => ThemeTitle.Text = ThemeDescription.IsFocused.ToString();
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

        private void ContentDialog_PreviewKeyDown(object sender, KeyEventArgs e)
        {
            if (ThemeDescription.IsFocused && (e.Key == Key.Escape || e.Key == Key.Enter))
            {
                e.Handled = true;
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}