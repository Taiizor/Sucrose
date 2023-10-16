using System.Windows;
using Wpf.Ui.Controls;
using SPMI = Sucrose.Portal.Manage.Internal;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// OtherHelp.xaml etkileşim mantığı
    /// </summary>
    public partial class OtherHelp : ContentDialog, IDisposable
    {
        public OtherHelp() : base(SPMI.ContentDialogService.GetContentPresenter())
        {
            InitializeComponent();
        }

        private void Report_Click(object sender, RoutedEventArgs e)
        {

        }

        private void WebPage_Click(object sender, RoutedEventArgs e)
        {

        }

        private void Repository_Click(object sender, RoutedEventArgs e)
        {

        }

        private void Discussions_Click(object sender, RoutedEventArgs e)
        {

        }

        private void Documentation_Click(object sender, RoutedEventArgs e)
        {

        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}