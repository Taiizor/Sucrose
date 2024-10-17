using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SMMRU = Sucrose.Memory.Manage.Readonly.Url;
using SHC = Skylark.Helper.Culture;
using SMMU = Sucrose.Manager.Manage.Update;
using SRER = Sucrose.Resources.Extension.Resources;
using SSCHV = Sucrose.Shared.Core.Helper.Version;

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
            Update.DescriptionText = string.Format(SRER.GetValue("Portal", "OtherAbout", "Update", "Description"), SMMU.Time.ToString(SHC.CurrentUI));

            HyperlinkButton Navigate = new()
            {
                NavigateUri = string.Format("{0}/v{1}", SMMRU.GitHubSucroseReleaseTag, Version),
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