using System.Windows;
using System.Windows.Controls;
using SPVCTC = Sucrose.Portal.Views.Controls.ThemeCard;

namespace Sucrose.Portal.Controls
{
    public class LibraryWrapPanel : WrapPanel, IDisposable
    {
        public static readonly DependencyProperty ItemMarginProperty = DependencyProperty.Register("ItemMargin", typeof(Thickness), typeof(LibraryWrapPanel), new FrameworkPropertyMetadata(new Thickness(0)));

        public Thickness ItemMargin
        {
            get => (Thickness)GetValue(ItemMarginProperty);
            set => SetValue(ItemMarginProperty, value);
        }

        public static readonly DependencyProperty MaxItemsPerRowProperty = DependencyProperty.Register("MaxItemsPerRow", typeof(int), typeof(LibraryWrapPanel), new FrameworkPropertyMetadata(int.MaxValue, MaxItemsPerRowPropertyChanged));

        public int MaxItemsPerRow
        {
            get => (int)GetValue(MaxItemsPerRowProperty);
            set => SetValue(MaxItemsPerRowProperty, value);
        }

        private static void MaxItemsPerRowPropertyChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
        {
            LibraryWrapPanel panel = d as LibraryWrapPanel;
            panel?.InvalidateMeasure();
        }

        protected override Size MeasureOverride(Size availableSize)
        {
            double x = 0;
            double y = 0;
            double rowHeight = 0;
            int itemsInCurrentRow = 0;

            if (InternalChildren.Count >= 0)
            {
                InternalChildren
                    .OfType<UIElement>()
                    .Where(Element => Element.Visibility != Visibility.Visible)
                    .ToList()
                    .ForEach(InternalChildren.Remove);

                foreach (UIElement child in InternalChildren)
                {
                    child.Measure(availableSize);
                    double childWidth = child.DesiredSize.Width + ItemMargin.Left + ItemMargin.Right;

                    if (x + childWidth > availableSize.Width || (MaxItemsPerRow > 0 && itemsInCurrentRow >= MaxItemsPerRow))
                    {
                        x = 0;
                        y += rowHeight;

                        rowHeight = 0;
                        itemsInCurrentRow = 0;
                    }

                    rowHeight = Math.Max(rowHeight, child.DesiredSize.Height + ItemMargin.Top + ItemMargin.Bottom);
                    x += childWidth;

                    itemsInCurrentRow++;
                }
            }

            y += rowHeight;

            return new Size(availableSize.Width, y);
        }

        protected override Size ArrangeOverride(Size finalSize)
        {
            double totalWidth = 0;
            double rowWidth = 0;
            double rowHeight = 0;
            int itemsInCurrentRow = 0;

            if (InternalChildren.Count >= 0)
            {
                for (int i = 0; i < InternalChildren.Count; i++)
                {
                    UIElement child = InternalChildren[i];
                    child.Measure(new Size(double.PositiveInfinity, finalSize.Height));

                    double childWidth = child.DesiredSize.Width + ItemMargin.Left + ItemMargin.Right;
                    double childHeight = child.DesiredSize.Height + ItemMargin.Top + ItemMargin.Bottom;

                    if (rowWidth + childWidth > finalSize.Width || (MaxItemsPerRow > 0 && itemsInCurrentRow >= MaxItemsPerRow))
                    {
                        double widthRatio = finalSize.Width / rowWidth;
                        double xOffset = 0;

                        for (int j = i - itemsInCurrentRow; j < i; j++)
                        {
                            UIElement rowChild = InternalChildren[j];
                            rowChild.Arrange(new Rect(new Point(xOffset + ItemMargin.Left, rowHeight + ItemMargin.Top), new Size(rowChild.DesiredSize.Width * widthRatio, rowChild.DesiredSize.Height)));
                            xOffset += (rowChild.DesiredSize.Width * widthRatio) + ItemMargin.Left + ItemMargin.Right;
                        }

                        totalWidth = Math.Max(totalWidth, rowWidth);
                        rowWidth = 0;
                        rowHeight += childHeight;
                        itemsInCurrentRow = 0;
                    }

                    rowWidth += childWidth;
                    itemsInCurrentRow++;
                }

                double remainingWidthRatio = finalSize.Width / rowWidth;
                double remainingXOffset = 0;

                for (int i = InternalChildren.Count - itemsInCurrentRow; i < InternalChildren.Count; i++)
                {
                    UIElement rowChild = InternalChildren[i];
                    rowChild.Arrange(new Rect(new Point(remainingXOffset + ItemMargin.Left, rowHeight + ItemMargin.Top), new Size(rowChild.DesiredSize.Width * remainingWidthRatio, rowChild.DesiredSize.Height)));
                    remainingXOffset += (rowChild.DesiredSize.Width * remainingWidthRatio) + ItemMargin.Left + ItemMargin.Right;
                }

                totalWidth = Math.Max(totalWidth, rowWidth);
            }

            return new Size(totalWidth, rowHeight);
        }

        public void Dispose()
        {
            if (InternalChildren.Count >= 0)
            {
                InternalChildren
                    .OfType<SPVCTC>()
                    .ToList()
                    .ForEach(Card => Card.Dispose());

                InternalChildren.Clear();
            }

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}