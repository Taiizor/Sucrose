using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SHC = Skylark.Helper.Culture;
using SMMM = Sucrose.Manager.Manage.Manager;
using SRER = Sucrose.Resources.Extension.Resources;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SMMU = Sucrose.Manager.Manage.Update;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// OtherAbout.xaml etkileşim mantığı
    /// </summary>
    public partial class OtherAbout : ContentDialog, IDisposable
    {
        public OtherAbout(ContentPresenter? contentPresenter) : base(contentPresenter)
        {
            InitializeComponent();
        }

        private void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            string Version = SSCHV.GetText();

            Update.TitleText = string.Format(SRER.GetValue("Portal", "OtherAbout", "Update"), Version);
            Update.DescriptionText = string.Format(SRER.GetValue("Portal", "OtherAbout", "Update", "Description"), SMMU.UpdateTime.ToString(SHC.CurrentUI));

            HyperlinkButton Navigate = new()
            {
                NavigateUri = string.Format("https://github.com/Taiizor/Sucrose/releases/tag/v{0}", Version),
                Foreground = SRER.GetResource<Brush>("AccentTextFillColorPrimaryBrush"),
                Content = SRER.GetValue("Portal", "OtherAbout", "Update", "Notes"),
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