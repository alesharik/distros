#include <liballoc.h>

/**  Durand's Amazing Super Duper Memory functions.  */

#define VERSION 	"1.1"
#define ALIGNMENT	16ul//4ul				///< This is the byte alignment that memory must be allocated on. IMPORTANT for GTK and other stuff.

#define ALIGN_TYPE		char ///unsigned char[16] /// unsigned short
#define ALIGN_INFO		sizeof(ALIGN_TYPE)*16	///< Alignment information is stored right before the pointer. This is the number of bytes of information stored there.


#define USE_CASE1
#define USE_CASE2
#define USE_CASE3
#define USE_CASE4
#define USE_CASE5


/** This macro will conveniently align our pointer upwards */
#define ALIGN( ptr )													\
		if ( ALIGNMENT > 1 )											\
		{																\
			uintptr_t diff;												\
			ptr = (void*)((uintptr_t)ptr + ALIGN_INFO);					\
			diff = (uintptr_t)ptr & (ALIGNMENT-1);						\
			if ( diff != 0 )											\
			{															\
				diff = ALIGNMENT - diff;								\
				ptr = (void*)((uintptr_t)ptr + diff);					\
			}															\
			*((ALIGN_TYPE*)((uintptr_t)ptr - ALIGN_INFO)) = 			\
				diff + ALIGN_INFO;										\
		}


#define UNALIGN( ptr )													\
		if ( ALIGNMENT > 1 )											\
		{																\
			uintptr_t diff = *((ALIGN_TYPE*)((uintptr_t)ptr - ALIGN_INFO));	\
			if ( diff < (ALIGNMENT + ALIGN_INFO) )						\
			{															\
				ptr = (void*)((uintptr_t)ptr - diff);					\
			}															\
		}

#define LIBALLOC_MAGIC	0xc001c0de
#define LIBALLOC_DEAD	0xdeaddead

static unsigned int l_pageSize  = 4096;			///< The size of an individual page. Set up in liballoc_init.
static unsigned int l_pageCount = 16;			///< The number of pages to request per chunk. Set up in liballoc_init.

// ***********   HELPER FUNCTIONS  *******************************

static void *liballoc_memset(void* s, int c, size_t n)
{
    unsigned int i;
    for ( i = 0; i < n ; i++)
        ((char*)s)[i] = c;

    return s;
}
static void* liballoc_memcpy(void* s1, const void* s2, size_t n)
{
    char *cdest;
    char *csrc;
    unsigned int *ldest = (unsigned int*)s1;
    unsigned int *lsrc  = (unsigned int*)s2;

    while ( n >= sizeof(unsigned int) )
    {
        *ldest++ = *lsrc++;
        n -= sizeof(unsigned int);
    }

    cdest = (char*)ldest;
    csrc  = (char*)lsrc;

    while ( n > 0 )
    {
        *cdest++ = *csrc++;
        n -= 1;
    }

    return s1;
}

// ***************************************************************

static struct liballoc_major *allocate_new_page(process_heap_inner *heap, unsigned int size )
{
    unsigned int st;
    struct liballoc_major *maj;

    // This is how much space is required.
    st  = size + sizeof(struct liballoc_major);
    st += sizeof(struct liballoc_minor);

    // Perfect amount of space?
    if ( (st % l_pageSize) == 0 )
        st  = st / (l_pageSize);
    else
        st  = st / (l_pageSize) + 1;
    // No, add the buffer.


    // Make sure it's >= the minimum size.
    if ( st < l_pageCount ) st = l_pageCount;

    maj = (struct liballoc_major*)liballoc_alloc( st );

    if ( maj == NULL )
    {
        heap->warning_count += 1;
        return NULL;	// uh oh, we ran out of memory.
    }

    maj->prev 	= NULL;
    maj->next 	= NULL;
    maj->pages 	= st;
    maj->size 	= st * l_pageSize;
    maj->usage 	= sizeof(struct liballoc_major);
    maj->first 	= NULL;

    heap->allocated += maj->size;
    return maj;
}

void *PREFIX(malloc)(process_heap_inner *heap, size_t req_size)
{
    int startedBet = 0;
    unsigned long long bestSize = 0;
    void *p = NULL;
    uintptr_t diff;
    struct liballoc_major *maj;
    struct liballoc_minor *min;
    struct liballoc_minor *new_min;
    unsigned long size = req_size;

    // For alignment, we adjust size so there's enough space to align.
    if ( ALIGNMENT > 1 )
    {
        size += ALIGNMENT + ALIGN_INFO;
    }
    // So, ideally, we really want an alignment of 0 or 1 in order
    // to save space.

    if ( size == 0 )
    {
        heap->warning_count += 1;
        return PREFIX(malloc)(heap,1);
    }


    if ( heap->root == NULL )
    {
        // This is the first time we are being used.
        heap->root = allocate_new_page( heap,size );
        if ( heap->root == NULL )
        {
            return NULL;
        }
    }

    // Now we need to bounce through every major and find enough space....

    maj = heap->root;
    startedBet = 0;

    // Start at the best bet....
    if ( heap->best_bet != NULL )
    {
        bestSize = heap->best_bet->size - heap->best_bet->usage;

        if ( bestSize > (size + sizeof(struct liballoc_minor)))
        {
            maj = heap->best_bet;
            startedBet = 1;
        }
    }

    while ( maj != NULL )
    {
        diff  = maj->size - maj->usage;
        // free memory in the block

        if ( bestSize < diff )
        {
            // Hmm.. this one has more memory then our bestBet. Remember!
            heap->best_bet = maj;
            bestSize = diff;
        }


#ifdef USE_CASE1

        // CASE 1:  There is not enough space in this major block.
        if ( diff < (size + sizeof( struct liballoc_minor )) )
        {
#ifdef DEBUG
            printf( "CASE 1: Insufficient space in block %x\n", maj);
			FLUSH();
#endif

            // Another major block next to this one?
            if ( maj->next != NULL )
            {
                maj = maj->next;		// Hop to that one.
                continue;
            }

            if ( startedBet == 1 )		// If we started at the best bet,
            {							// let's start all over again.
                maj = heap->root;
                startedBet = 0;
                continue;
            }

            // Create a new major block next to this one and...
            maj->next = allocate_new_page( heap,size );	// next one will be okay.
            if ( maj->next == NULL ) break;			// no more memory.
            maj->next->prev = maj;
            maj = maj->next;

            // .. fall through to CASE 2 ..
        }

#endif

#ifdef USE_CASE2

        // CASE 2: It's a brand new block.
        if ( maj->first == NULL )
        {
            maj->first = (struct liballoc_minor*)((uintptr_t)maj + sizeof(struct liballoc_major) );


            maj->first->magic 		= LIBALLOC_MAGIC;
            maj->first->prev 		= NULL;
            maj->first->next 		= NULL;
            maj->first->block 		= maj;
            maj->first->size 		= size;
            maj->first->req_size 	= req_size;
            maj->usage 	+= size + sizeof( struct liballoc_minor );


            heap->inuse += size;


            p = (void*)((uintptr_t)(maj->first) + sizeof( struct liballoc_minor ));

            ALIGN( p );
            return p;
        }

#endif

#ifdef USE_CASE3

        // CASE 3: Block in use and enough space at the start of the block.
        diff =  (uintptr_t)(maj->first);
        diff -= (uintptr_t)maj;
        diff -= sizeof(struct liballoc_major);

        if ( diff >= (size + sizeof(struct liballoc_minor)) )
        {
            // Yes, space in front. Squeeze in.
            maj->first->prev = (struct liballoc_minor*)((uintptr_t)maj + sizeof(struct liballoc_major) );
            maj->first->prev->next = maj->first;
            maj->first = maj->first->prev;

            maj->first->magic 	= LIBALLOC_MAGIC;
            maj->first->prev 	= NULL;
            maj->first->block 	= maj;
            maj->first->size 	= size;
            maj->first->req_size 	= req_size;
            maj->usage 			+= size + sizeof( struct liballoc_minor );

            heap->inuse += size;

            p = (void*)((uintptr_t)(maj->first) + sizeof( struct liballoc_minor ));
            ALIGN( p );
            return p;
        }

#endif


#ifdef USE_CASE4

        // CASE 4: There is enough space in this block. But is it contiguous?
        min = maj->first;

        // Looping within the block now...
        while ( min != NULL )
        {
            // CASE 4.1: End of minors in a block. Space from last and end?
            if ( min->next == NULL )
            {
                // the rest of this block is free...  is it big enough?
                diff = (uintptr_t)(maj) + maj->size;
                diff -= (uintptr_t)min;
                diff -= sizeof( struct liballoc_minor );
                diff -= min->size;
                // minus already existing usage..

                if ( diff >= (size + sizeof( struct liballoc_minor )) )
                {
                    // yay....
                    min->next = (struct liballoc_minor*)((uintptr_t)min + sizeof( struct liballoc_minor ) + min->size);
                    min->next->prev = min;
                    min = min->next;
                    min->next = NULL;
                    min->magic = LIBALLOC_MAGIC;
                    min->block = maj;
                    min->size = size;
                    min->req_size = req_size;
                    maj->usage += size + sizeof( struct liballoc_minor );

                    heap->inuse += size;

                    p = (void*)((uintptr_t)min + sizeof( struct liballoc_minor ));
                    ALIGN( p );
                    return p;
                }
            }



            // CASE 4.2: Is there space between two minors?
            if ( min->next != NULL )
            {
                // is the difference between here and next big enough?
                diff  = (uintptr_t)(min->next);
                diff -= (uintptr_t)min;
                diff -= sizeof( struct liballoc_minor );
                diff -= min->size;
                // minus our existing usage.

                if ( diff >= (size + sizeof( struct liballoc_minor )) )
                {
                    // yay......
                    new_min = (struct liballoc_minor*)((uintptr_t)min + sizeof( struct liballoc_minor ) + min->size);

                    new_min->magic = LIBALLOC_MAGIC;
                    new_min->next = min->next;
                    new_min->prev = min;
                    new_min->size = size;
                    new_min->req_size = req_size;
                    new_min->block = maj;
                    min->next->prev = new_min;
                    min->next = new_min;
                    maj->usage += size + sizeof( struct liballoc_minor );

                    heap->inuse += size;

                    p = (void*)((uintptr_t)new_min + sizeof( struct liballoc_minor ));
                    ALIGN( p );
                    return p;
                }
            }	// min->next != NULL

            min = min->next;
        } // while min != NULL ...


#endif

#ifdef USE_CASE5

        // CASE 5: Block full! Ensure next block and loop.
        if ( maj->next == NULL )
        {
            if ( startedBet == 1 )
            {
                maj = heap->root;
                startedBet = 0;
                continue;
            }

            // we've run out. we need more...
            maj->next = allocate_new_page( heap, size );		// next one guaranteed to be okay
            if ( maj->next == NULL ) break;			//  uh oh,  no more memory.....
            maj->next->prev = maj;

        }

#endif

        maj = maj->next;
    } // while (maj != NULL)
    return NULL;
}

void PREFIX(free)(process_heap_inner *heap, void *ptr)
{
    struct liballoc_minor *min;
    struct liballoc_major *maj;

    if ( ptr == NULL )
    {
        heap->warning_count += 1;
        return;
    }

    UNALIGN( ptr );


    min = (struct liballoc_minor*)((uintptr_t)ptr - sizeof( struct liballoc_minor ));


    if ( min->magic != LIBALLOC_MAGIC )
    {
        heap->error_count += 1;

        // Check for overrun errors. For all bytes of LIBALLOC_MAGIC
        if (
                ((min->magic & 0xFFFFFF) == (LIBALLOC_MAGIC & 0xFFFFFF)) ||
                ((min->magic & 0xFFFF) == (LIBALLOC_MAGIC & 0xFFFF)) ||
                ((min->magic & 0xFF) == (LIBALLOC_MAGIC & 0xFF))
                )
        {
            heap->possible_overruns += 1;
        }


        if ( min->magic == LIBALLOC_DEAD )
        {
#if defined DEBUG || defined INFO
            printf( "liballoc: ERROR: multiple PREFIX(free)() attempt on %x from %x.\n",
									ptr,
									__builtin_return_address(0) );
			FLUSH();
#endif
        }
        else
        {
#if defined DEBUG || defined INFO
            printf( "liballoc: ERROR: Bad PREFIX(free)( %x ) called from %x\n",
								ptr,
								__builtin_return_address(0) );
			FLUSH();
#endif
        }

        // being lied to...
        return;
    }

#ifdef DEBUG
    printf( "liballoc: %x PREFIX(free)( %x ): ",
				__builtin_return_address( 0 ),
				ptr );
	FLUSH();
#endif


    maj = min->block;

    heap->inuse -= min->size;

    maj->usage -= (min->size + sizeof( struct liballoc_minor ));
    min->magic  = LIBALLOC_DEAD;		// No mojo.

    if ( min->next != NULL ) min->next->prev = min->prev;
    if ( min->prev != NULL ) min->prev->next = min->next;

    if ( min->prev == NULL ) maj->first = min->next;
    // Might empty the block. This was the first
    // minor.


    // We need to clean up after the majors now....

    if ( maj->first == NULL )	// Block completely unused.
    {
        if ( heap->root == maj ) heap->root = maj->next;
        if ( heap->best_bet == maj ) heap->best_bet = NULL;
        if ( maj->prev != NULL ) maj->prev->next = maj->next;
        if ( maj->next != NULL ) maj->next->prev = maj->prev;
        heap->allocated -= maj->size;

        liballoc_free( maj, maj->pages );
    }
    else
    {
        if ( heap->best_bet != NULL )
        {
            int bestSize = heap->best_bet->size  - heap->best_bet->usage;
            int majSize = maj->size - maj->usage;

            if ( majSize > bestSize ) heap->best_bet = maj;
        }

    }
}

void* PREFIX(calloc)(process_heap_inner *heap, size_t nobj, size_t size)
{
    int real_size;
    void *p;

    real_size = nobj * size;

    p = PREFIX(malloc)( heap, real_size );

    liballoc_memset( p, 0, real_size );

    return p;
}



void*   PREFIX(realloc)(process_heap_inner *heap, void *p, size_t size)
{
    void *ptr;
    struct liballoc_minor *min;
    unsigned int real_size;

    // Honour the case of size == 0 => free old and return NULL
    if ( size == 0 )
    {
        PREFIX(free)( heap, p );
        return NULL;
    }

    // In the case of a NULL pointer, return a simple malloc.
    if ( p == NULL ) return PREFIX(malloc)( heap, size );

    // Unalign the pointer if required.
    ptr = p;
    UNALIGN(ptr);

    min = (struct liballoc_minor*)((uintptr_t)ptr - sizeof( struct liballoc_minor ));

    // Ensure it is a valid structure.
    if ( min->magic != LIBALLOC_MAGIC )
    {
        heap->error_count += 1;

        // Check for overrun errors. For all bytes of LIBALLOC_MAGIC
        if (
                ((min->magic & 0xFFFFFF) == (LIBALLOC_MAGIC & 0xFFFFFF)) ||
                ((min->magic & 0xFFFF) == (LIBALLOC_MAGIC & 0xFFFF)) ||
                ((min->magic & 0xFF) == (LIBALLOC_MAGIC & 0xFF))
                )
        {
            heap->possible_overruns += 1;
#if defined DEBUG || defined INFO
            printf( "liballoc: ERROR: Possible 1-3 byte overrun for magic %x != %x\n",
									min->magic,
									LIBALLOC_MAGIC );
				FLUSH();
#endif
        }


        if ( min->magic == LIBALLOC_DEAD )
        {
#if defined DEBUG || defined INFO
            printf( "liballoc: ERROR: multiple PREFIX(free)() attempt on %x from %x.\n",
										ptr,
										__builtin_return_address(0) );
				FLUSH();
#endif
        }
        else
        {
#if defined DEBUG || defined INFO
            printf( "liballoc: ERROR: Bad PREFIX(free)( %x ) called from %x\n",
									ptr,
									__builtin_return_address(0) );
				FLUSH();
#endif
        }

        // being lied to...
        return NULL;
    }

    // Definitely a memory block.

    real_size = min->req_size;

    if ( real_size >= size )
    {
        min->req_size = size;
        return p;
    }

    // If we got here then we're reallocating to a block bigger than us.
    ptr = PREFIX(malloc)( heap, size );					// We need to allocate new memory
    liballoc_memcpy( ptr, p, real_size );
    PREFIX(free)( heap, p );

    return ptr;
}




