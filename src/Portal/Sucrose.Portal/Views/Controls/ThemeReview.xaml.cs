using System.IO;
using System.Windows;
using Wpf.Ui.Controls;
using SECNT = Skylark.Enum.ClearNumericType;
using SEMST = Skylark.Enum.ModeStorageType;
using SEST = Skylark.Enum.StorageType;
using SHC = Skylark.Helper.Culture;
using SHN = Skylark.Helper.Numeric;
using SPEIL = Sucrose.Portal.Extension.ImageLoader;
using SPMI = Sucrose.Portal.Manage.Internal;
using SSESSE = Skylark.Standard.Extension.Storage.StorageExtension;
using SSSHS = Sucrose.Shared.Space.Helper.Size;
using SSSSS = Skylark.Struct.Storage.StorageStruct;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeReview.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeReview : ContentDialog, IDisposable
    {
        private readonly SPEIL Loader = new();
        internal string Theme = string.Empty;
        internal SSTHI Info = new();

        public ThemeReview() : base(SPMI.ContentDialogService.GetContentPresenter())
        {
            InitializeComponent();
        }

        private string Size(string Path)
        {
            SSSSS Data = SSESSE.AutoConvert(SSSHS.Calc(Path), SEST.Byte, SEMST.Palila);

            return $"{SHN.Numeral(Data.Value, true, true, 1, '0', SECNT.None)} {Data.Short}";
        }

        private void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            ThemeTitle.Text = Info.Title;
            ThemeDescription.Text = Info.Description;

            ThemeTypeName.Text = Info.Type.ToString();

            ThemeAuthorName.Text = Info.Author;

            ThemeSizeTotal.Text = Size(Theme);

            ThemeVersionText.Text = $"{Info.Version} ({Info.AppVersion})";

            DateTime CreationTime = Directory.GetCreationTime(Theme);

            ThemeCreateDate.Text = CreationTime.ToString(SHC.CurrentUI);

            if (string.IsNullOrEmpty(Info.Contact))
            {
                ThemeContact.Visibility = Visibility.Collapsed;
            }
            else
            {
                if (SSTHV.IsMail(Info.Contact))
                {
                    ThemeContact.NavigateUri = $"mailto:{Info.Contact}";
                }
                else
                {
                    ThemeContact.NavigateUri = Info.Contact;
                }
            }

            string ImagePath = Path.Combine(Theme, Info.Thumbnail);

            if (File.Exists(ImagePath))
            {
                ThemeThumbnail.Source = Loader.LoadOptimal(ImagePath, true, 600);
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}