using Sucrose.Portal.ViewModels;
using Wpf.Ui.Controls;

namespace Sucrose.Portal.Views.Pages
{
    /// <summary>
    /// Library.xaml etkileşim mantığı
    /// </summary>
    public partial class Library : INavigableView<LibraryViewModel>
    {
        public LibraryViewModel ViewModel { get; }

        public Library()
        {
            InitializeComponent();
        }
    }
}