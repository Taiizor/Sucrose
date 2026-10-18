using System.Windows;
using Wpf.Ui.Controls;
using SHC = Skylark.Helper.Culture;
using SMMRU = Sucrose.Memory.Manage.Readonly.Url;
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
        public OtherAbout(ContentDialogHost? contentDialogHost) : base(contentDialogHost)
        {
            InitializeComponent();
        }

        private void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            string Version = SSCHV.GetText();

            UpdateTitle.Text = string.Format(SRER.GetValue("Portal", "OtherAbout", "Update"), Version);
            UpdateDescription.Text = string.Format(SRER.GetValue("Portal", "OtherAbout", "Update", "Description"), SMMU.Time.ToString(SHC.CurrentUI));

            UpdateNotes.NavigateUri = string.Format("{0}/v{1}", SMMRU.GitHubSucroseReleaseTag, Version);
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}