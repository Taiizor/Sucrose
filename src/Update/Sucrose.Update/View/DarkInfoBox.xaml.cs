using System.Media;
using System.Windows;
using SSDEUT = Sucrose.Shared.Dependency.Enum.UpdateType;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SSSHD = Sucrose.Shared.Space.Helper.Dark;
using SWHWI = Skylark.Wing.Helper.WindowInterop;

namespace Sucrose.Update.View
{
    /// <summary>
    /// DarkInfoBox.xaml etkileşim mantığı
    /// </summary>
    public partial class DarkInfoBox : Window
    {
        internal DarkInfoBox(SSDEUT Type)
        {
            InitializeComponent();

            switch (Type)
            {
                case SSDEUT.Empty:
                    Info_Image.Visibility = Visibility.Visible;
                    break;
                case SSDEUT.Update:
                    Success_Image.Visibility = Visibility.Visible;
                    break;
                case SSDEUT.Status:
                    Warn_Image.Visibility = Visibility.Visible;
                    break;
                case SSDEUT.Network:
                    Error_Image.Visibility = Visibility.Visible;
                    break;
                case SSDEUT.Updating:
                    Success_Image.Visibility = Visibility.Visible;
                    break;
                case SSDEUT.Releases:
                    Warn_Image.Visibility = Visibility.Visible;
                    break;
                default:
                    Sad_Image.Visibility = Visibility.Visible;
                    break;
            }

            SystemSounds.Asterisk.Play();

            SourceInitialized += DarkInfoBox_SourceInitialized;

            Text_Message.Text = SSRER.GetValue("Update", "InfoBox", "TextMessage", $"{Type}");
        }

        private void CloseButton_Click(object sender, RoutedEventArgs e)
        {
            Close();
        }

        private void DarkInfoBox_SourceInitialized(object sender, EventArgs e)
        {
            SSSHD.Apply(SWHWI.Handle(this));
        }
    }
}