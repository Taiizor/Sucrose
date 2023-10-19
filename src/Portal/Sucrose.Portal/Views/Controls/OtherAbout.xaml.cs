using System.Windows;
using System.Windows.Input;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SHC = Skylark.Helper.Culture;
using SMMM = Sucrose.Manager.Manage.Manager;
using SPMI = Sucrose.Portal.Manage.Internal;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// OtherAbout.xaml etkileşim mantığı
    /// </summary>
    public partial class OtherAbout : ContentDialog, IDisposable
    {
        public OtherAbout() : base(SPMI.ContentDialogService.GetContentPresenter())
        {
            InitializeComponent();
        }

        private void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            string Version = SSCHV.GetText();

            Update.TitleText = string.Format(SSRER.GetValue("Portal", "OtherAbout", "Update"), Version);
            Update.DescriptionText = string.Format(SSRER.GetValue("Portal", "OtherAbout", "Update", "Description"), SMMM.UpdateTime.ToString(SHC.CurrentUI));

            HyperlinkButton Navigate = new()
            {
                NavigateUri = string.Format("https://github.com/Taiizor/Sucrose/releases/tag/v{0}", Version),
                Foreground = SSRER.GetResource<Brush>("AccentTextFillColorPrimaryBrush"),
                Content = SSRER.GetValue("Portal", "OtherAbout", "Update", "Notes"),
                Icon = new SymbolIcon(SymbolRegular.Notepad24),
                Appearance = ControlAppearance.Transparent,
                BorderBrush = Brushes.Transparent,
                Cursor = Cursors.Hand,
                FontSize = 14
            };

            Update.HeaderFrame = Navigate;
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}