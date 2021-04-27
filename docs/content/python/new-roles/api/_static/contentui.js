
$(function() {
    /*
     * Right column logic
     */
    if ($(".right-col").length) {
        $(".right-col").after('<div class="clear"></div>');
        $(".right-col").parents('body').addClass('with-columns');
    }

    /**
     * Toggle logic
     */
    $('.toggle-content').hide()
    $('.toggle-header').click(function () {
        $(this).toggleClass("open");
        $(this).next('.toggle-content').toggle('400');
    })
    
    /**
     * Dynamic multiple content block.
     */
    var top_sel = {}

    $('div.content-tabs').each(function() {
        var contenttab_sel = $('<ul />', { class: "contenttab-selector" });
        var i = 0;

        if ($(this).hasClass('right-col')){
            contenttab_sel.addClass('in-right-col');
        }

        $('.tab-content', this).each(function() {
            var sel_item = $('<li />', {
                class: $(this).attr('id'),
                text: $(this).find('.tab-title').text()
            });
            $(this).find('.tab-title').remove();
            if (i++) {
                $(this).hide();
            } else {
                sel_item.addClass('selected');
            }
            contenttab_sel.append(sel_item);
            $(this).addClass('contenttab');
        });

        $('.tab-content', this).eq(0).before(contenttab_sel);
        contenttab_sel = null;
        i = null;
    });


    $('.contenttab-selector li').click(function(evt) {
        evt.preventDefault();

        if ($(this).parents('.in-right-col').length){
            var tabsblock = $('.right-col');
        }else{
            var tabsblock = $(this).parents('.content-tabs');
        }

        var sel_class = $(this).attr('class');
        $('div.contenttab',tabsblock).hide();
        $('div#' + sel_class,tabsblock).show();

        $('ul.contenttab-selector li', tabsblock).removeClass('selected');
        $('ul.contenttab-selector li.' + sel_class, tabsblock).addClass('selected');

        sel_class = null;
    });

});

