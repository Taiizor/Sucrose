using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ExpanderCard.xaml etkileşim mantığı
    /// </summary>
    public partial class ExpanderCard : UserControl
    {
        public bool Expandable = true;
        public bool IsExpand = false;

        public ExpanderCard()
        {
            InitializeComponent();

            if (Expandable)
            {
                if (IsExpand)
                {
                    ExpandUp.Visibility = Visibility.Visible;
                    ExpandDown.Visibility = Visibility.Hidden;
                    FooterControl.Visibility = Visibility.Visible;
                }
                else
                {
                    ExpandUp.Visibility = Visibility.Hidden;
                    ExpandDown.Visibility = Visibility.Visible;
                    FooterControl.Visibility = Visibility.Hidden;
                }

                Footer.Margin = new(Body.Margin.Left + LeftIcon.Width, 0, 0, 0);
            }
            else
            {
                Grider.ColumnDefinitions.RemoveAt(3);
                ExpandUp.Visibility = Visibility.Hidden;
                ExpandDown.Visibility = Visibility.Hidden;
                FooterControl.Visibility = Visibility.Hidden;
            }
        }

        public Thickness FooterFrameMargin
        {
            get { return Footer.Margin; }
            set { Footer.Margin = value; }
        }

        public object HeaderFrame
        {
            get { return Header.Margin; }
            set { Header.Content = value; }
        }

        public object FooterCard
        {
            get { return Footer.Margin; }
            set { Footer.Content = value; }
        }

        private void Expand_Click(object sender, RoutedEventArgs e)
        {
            if (Expandable)
            {
                IsExpand = !IsExpand;

                if (IsExpand)
                {
                    ExpandUp.Visibility = Visibility.Visible;
                    ExpandDown.Visibility = Visibility.Hidden;
                    FooterControl.Visibility = Visibility.Visible;
                }
                else
                {
                    ExpandUp.Visibility = Visibility.Hidden;
                    ExpandDown.Visibility = Visibility.Visible;
                    FooterControl.Visibility = Visibility.Hidden;
                }
            }
        }

        private void Card_MouseLeftButtonDown(object sender, MouseButtonEventArgs e)
        {
            if (Expandable)
            {
                Expand_Click(null, null);
            }
        }
    }
}
