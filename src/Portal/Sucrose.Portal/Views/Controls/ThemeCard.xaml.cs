using Sucrose.Shared.Theme.Helper;
using System.IO;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media.Imaging;
using Wpf.Ui.Animations;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SHA = Skylark.Helper.Adaptation;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeCard.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeCard : UserControl
    {
        private SSTHI ThemeInfo = null;
        private string ThemePath = null;

        internal ThemeCard(string Theme, SSTHI Info)
        {
            ThemeInfo = Info;
            ThemePath = Theme;
            InitializeComponent();

            ThemeTitle.Text = SHA.Cut(Info.Title, 35);
            ThemeDescription.Text = SHA.Cut(Info.Description, 30);

            Imagine.ImageSource = new BitmapImage(new Uri(Path.Combine(Theme, Info.Thumbnail)));
        }

        private void ThemeCard_MouseEnter(object sender, MouseEventArgs e)
        {
            //ThemeTitle.Text = "Enter";
            //TransitionAnimationProvider.ApplyTransition(ThemeButton, TransitionType.FadeIn, 5);
        }

        private void ThemeCard_MouseLeave(object sender, MouseEventArgs e)
        {
            //ThemeTitle.Text = "Leave";
            //TransitionAnimationProvider.ApplyTransition(ThemeButton, TransitionType.FadeInWithSlide, 5);
        }
    }
}