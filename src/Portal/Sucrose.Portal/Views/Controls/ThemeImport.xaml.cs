using System.Windows;
using Wpf.Ui.Controls;
using SPMI = Sucrose.Portal.Manage.Internal;
using SSDECT = Sucrose.Shared.Dependency.Enum.CompatibilityType;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeImport.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeImport : ContentDialog, IDisposable
    {
        internal List<SSDECT> Types = new();
        internal string Info = string.Empty;

        public ThemeImport() : base(SPMI.ContentDialogService.GetContentPresenter())
        {
            InitializeComponent();
        }

        private void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            ThemeImportInfo.Text = Info;

            if (Types.Any())
            {
                //Legends (hata kodlarının anlamını burada kısaca ya da basitçe açıkla)

                //Test: Test açıklaması
                //Test2: Test2 açıklaması
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}