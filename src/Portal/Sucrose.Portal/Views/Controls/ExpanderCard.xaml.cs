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
        private bool _Expandable { get; set; } = true;

        private bool _IsExpand { get; set; } = false;

        public ExpanderCard()
        {
            InitializeComponent();
            UpdateExpandState();
        }

        public Thickness FooterFrameMargin
        {
            get => Footer.Margin;
            set => Footer.Margin = value;
        }

        public object HeaderFrame
        {
            get => Header.Margin;
            set => Header.Content = value;
        }

        public object FooterCard
        {
            get => Footer.Margin;
            set => Footer.Content = value;
        }

        public bool IsExpand
        {
            get => _IsExpand;
            set
            {
                _IsExpand = value;
                UpdateExpandState();
            }
        }

        public bool Expandable
        {
            get => _Expandable;
            set
            {
                _Expandable = value;

                if (IsExpand != value)
                {
                    IsExpand = false;
                }

                UpdateExpandState();
            }
        }

        private void UpdateExpandState()
        {
            if (Expandable)
            {
                if (Grider.ColumnDefinitions.Count < 4)
                {
                    ColumnDefinition NewColumn = new()
                    {
                        Width = new(1, GridUnitType.Auto)
                    };

                    Grider.ColumnDefinitions.Insert(3, NewColumn);
                }

                ExpandUp.Visibility = IsExpand ? Visibility.Visible : Visibility.Hidden;
                ExpandDown.Visibility = IsExpand ? Visibility.Hidden : Visibility.Visible;
                FooterControl.Visibility = IsExpand ? Visibility.Visible : Visibility.Hidden;

                FooterFrameMargin = new Thickness(Body.Margin.Left + LeftIcon.Width, 0, 0, 0);
            }
            else
            {
                if (Grider.ColumnDefinitions.Count > 3)
                {
                    Grider.ColumnDefinitions.RemoveAt(3);
                }

                ExpandUp.Visibility = Visibility.Hidden;
                ExpandDown.Visibility = Visibility.Hidden;
                FooterControl.Visibility = Visibility.Hidden;
            }
        }

        private void ToggleExpandState()
        {
            if (Expandable)
            {
                IsExpand = !IsExpand;
                UpdateExpandState();
            }
        }

        private void Expand_Click(object sender, RoutedEventArgs e)
        {
            ToggleExpandState();
        }

        private void Card_MouseLeftButtonDown(object sender, MouseButtonEventArgs e)
        {
            ToggleExpandState();
        }
    }
}
