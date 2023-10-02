using System.IO;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SECNT = Skylark.Enum.ClearNumericType;
using SEMST = Skylark.Enum.ModeStorageType;
using SEST = Skylark.Enum.StorageType;
using SHC = Skylark.Helper.Culture;
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
    /// DisplayPreferences.xaml etkileşim mantığı
    /// </summary>
    public partial class DisplayPreferences : ContentDialog, IDisposable
    {

        public DisplayPreferences() : base(SPMI.ContentDialogService.GetContentPresenter())
        {
            InitializeComponent();
        }

        private void MonitorClicked(object sender, MouseButtonEventArgs e)
        {
            Border border = sender as Border;
            if (border != null)
            {
                // Seçili monitörün border rengini değiştirme
                border.BorderBrush = Brushes.CornflowerBlue;

                // Diğer monitörlerin border rengini sıfırlama
                WrapPanel wrapPanel = border.Parent as WrapPanel;
                if (wrapPanel != null)
                {
                    foreach (var item in wrapPanel.Children)
                    {
                        if (item is Border otherBorder && otherBorder != border)
                        {
                            otherBorder.BorderBrush = Brushes.Black;
                        }
                    }
                }
            }
        }

        private void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}