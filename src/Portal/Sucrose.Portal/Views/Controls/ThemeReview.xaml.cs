using System.Windows;
using Wpf.Ui.Controls;
using SECNT = Skylark.Enum.ClearNumericType;
using SEMST = Skylark.Enum.ModeStorageType;
using SEST = Skylark.Enum.StorageType;
using SHN = Skylark.Helper.Numeric;
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

            if (string.IsNullOrEmpty(Info.Contact))
            {
                ThemeContact.Visibility = Visibility.Collapsed;
            }
            else
            {
                if (SSTHV.IsMail(Info.Contact))
                {
                    Info.Contact = $"mailto:{Info.Contact}";
                }

                ThemeContact.NavigateUri = Info.Contact;
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}